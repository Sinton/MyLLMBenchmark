use super::helpers::{
    api_url, classify_model, parse_vision_sample, request_timeout, rerank_prompt_for_log,
    VisionSample,
};
use super::outcome::{
    estimate_tokens, raw_usage_from_value, usage_from_value, RequestOutcome, RequestUnits,
    TokenUsage,
};
use super::protocol::{ensure_success, map_reqwest_error, ModelsResponse, RealProviderProtocol};
use super::providers::{
    anthropic, embedding_openai as embedding, gemini, jina_rerank as rerank,
    openai_compatible as chat, openai_responses as responses, vision_openai as vision,
};
use super::streaming::collect_streaming_response;
use crate::domain::workload::WorkloadConfig;
use crate::models::{DatasetSample, DiscoveredModel, ProviderConnectionConfig};
use reqwest::Client;
use std::time::Duration;
use tokio::time::Instant;
#[derive(Clone)]
pub struct RealProviderClient {
    pub(super) client: Client,
}

impl RealProviderClient {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::builder().build()?,
        })
    }

    pub async fn list_models(
        &self,
        config: &ProviderConnectionConfig,
    ) -> anyhow::Result<Vec<DiscoveredModel>> {
        let protocol = RealProviderProtocol::from_interface_type(&config.interface_type)
            .unwrap_or(RealProviderProtocol::OpenAICompatible);
        if protocol == RealProviderProtocol::Gemini {
            return self.list_gemini_models(config).await;
        }
        let url = api_url(&config.base_url, "models");
        let mut request = self.with_protocol_auth(self.client.get(url), config, protocol);
        if protocol == RealProviderProtocol::Anthropic {
            request = request.header("anthropic-version", "2023-06-01");
        }
        let response = request
            .timeout(Duration::from_secs(20))
            .send()
            .await
            .map_err(map_reqwest_error)?;

        ensure_success(response.status(), "models list")?;
        let payload = response.json::<ModelsResponse>().await?;
        Ok(payload
            .data
            .into_iter()
            .map(|model| classify_model(&model.id))
            .collect())
    }

    async fn list_gemini_models(
        &self,
        config: &ProviderConnectionConfig,
    ) -> anyhow::Result<Vec<DiscoveredModel>> {
        let url = api_url(&config.base_url, "models");
        let response = self
            .with_protocol_auth(
                self.client
                    .get(url)
                    .header(reqwest::header::USER_AGENT, "MyLLMBenchmark"),
                config,
                RealProviderProtocol::Gemini,
            )
            .timeout(Duration::from_secs(20))
            .send()
            .await
            .map_err(map_reqwest_error)?;

        ensure_success(response.status(), "models list")?;
        let payload = response.json::<serde_json::Value>().await?;
        Ok(payload
            .get("models")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
            .filter_map(|model| {
                model
                    .get("name")
                    .and_then(|value| value.as_str())
                    .map(|name| name.trim_start_matches("models/").to_string())
            })
            .map(|name| classify_model(&name))
            .collect())
    }

    async fn chat_completion(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        prompt: &str,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        if workload.streaming {
            self.chat_completion_streaming(config, model, prompt, workload, request_timeout_seconds)
                .await
        } else {
            self.chat_completion_non_streaming(
                config,
                model,
                prompt,
                workload,
                request_timeout_seconds,
            )
            .await
        }
    }

    pub(super) async fn text_generation(
        &self,
        config: &ProviderConnectionConfig,
        protocol: RealProviderProtocol,
        model: &str,
        prompt: &str,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        match protocol {
            RealProviderProtocol::OpenAICompatible => {
                self.chat_completion(config, model, prompt, workload, request_timeout_seconds)
                    .await
            }
            RealProviderProtocol::OpenAIResponses => {
                self.openai_response(
                    config,
                    model,
                    prompt,
                    workload,
                    request_timeout_seconds,
                    None,
                )
                .await
            }
            RealProviderProtocol::Anthropic => {
                self.anthropic_message(
                    config,
                    model,
                    prompt,
                    workload,
                    request_timeout_seconds,
                    None,
                )
                .await
            }
            RealProviderProtocol::Gemini => {
                self.gemini_generate_content(
                    config,
                    model,
                    prompt,
                    workload,
                    request_timeout_seconds,
                    None,
                )
                .await
            }
        }
    }

    pub(super) async fn embedding(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        inputs: Vec<String>,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        let started = Instant::now();
        let input_tokens = inputs.iter().map(|item| estimate_tokens(item)).sum::<i64>();
        let text_count = inputs.len() as i64;
        let prompt_for_log = inputs.join("\n---\n");
        let body = embedding::embeddings_body(model, inputs);
        let response = match self
            .with_auth(
                self.client.post(api_url(&config.base_url, "embeddings")),
                config,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt_for_log),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt_for_log),
                None,
                None,
            );
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt_for_log), None, None)
            }
        };
        let usage = usage_from_value(&payload).unwrap_or(TokenUsage {
            input_tokens,
            output_tokens: 0,
            total_tokens: input_tokens,
        });
        RequestOutcome::success_with_units(
            started.elapsed(),
            Duration::ZERO,
            usage,
            RequestUnits {
                batch_size: text_count,
                text_count,
                ..RequestUnits::default()
            },
        )
        .with_body(
            Some(prompt_for_log),
            Some(format!(
                "Embedding 返回 {text_count} 条向量，向量正文未保存。"
            )),
            raw_usage_from_value(&payload),
        )
    }

    pub(super) async fn rerank(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        query: String,
        documents: Vec<String>,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        let started = Instant::now();
        let documents_per_query = documents.len() as i64;
        let query_for_log = query.clone();
        let documents_for_log = documents.clone();
        let prompt_for_log = rerank_prompt_for_log(&query_for_log, &documents_for_log);
        let input_tokens = estimate_tokens(&query)
            + documents
                .iter()
                .map(|item| estimate_tokens(item))
                .sum::<i64>();
        let body = rerank::rerank_body(model, query, documents, workload);
        let response = match self
            .with_auth(
                self.client.post(api_url(&config.base_url, "rerank")),
                config,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt_for_log),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt_for_log),
                None,
                None,
            );
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt_for_log), None, None)
            }
        };
        let result_count = payload
            .get("results")
            .and_then(|value| value.as_array())
            .map(|items| items.len())
            .unwrap_or(0);
        RequestOutcome::success_with_units(
            started.elapsed(),
            Duration::ZERO,
            TokenUsage {
                input_tokens,
                output_tokens: 0,
                total_tokens: input_tokens,
            },
            RequestUnits {
                documents_per_query,
                pair_count: documents_per_query,
                ..RequestUnits::default()
            },
        )
        .with_body(
            Some(prompt_for_log),
            Some(format!(
                "Rerank 返回 {result_count} 条结果，候选文档 {documents_per_query} 条。"
            )),
            raw_usage_from_value(&payload),
        )
    }

    async fn chat_completion_non_streaming(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        prompt: &str,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        let started = Instant::now();
        let body = chat::completion_body(model, prompt, workload, false, 0.7);
        let response = match self
            .with_auth(
                self.client
                    .post(api_url(&config.base_url, "chat/completions")),
                config,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt.to_string()),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt.to_string()),
                None,
                None,
            );
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt.to_string()), None, None)
            }
        };
        let output = payload
            .pointer("/choices/0/message/content")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        RequestOutcome::success(
            started.elapsed(),
            started.elapsed(),
            usage_from_value(&payload).unwrap_or_else(|| TokenUsage::estimated(prompt, &output)),
        )
        .with_body(
            Some(prompt.to_string()),
            Some(output),
            raw_usage_from_value(&payload),
        )
    }

    async fn chat_completion_streaming(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        prompt: &str,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        let started = Instant::now();
        let body = chat::streaming_completion_body(model, prompt, workload);
        let response = match self
            .with_auth(
                self.client
                    .post(api_url(&config.base_url, "chat/completions")),
                config,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt.to_string()),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt.to_string()),
                None,
                None,
            );
        }

        collect_streaming_response(
            response,
            RealProviderProtocol::OpenAICompatible,
            prompt,
            started,
            RequestUnits::default(),
        )
        .await
    }

    async fn openai_response(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        prompt: &str,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
        vision_sample: Option<&VisionSample>,
    ) -> RequestOutcome {
        let started = Instant::now();
        let units = RequestUnits {
            image_count: vision_sample
                .map(|sample| sample.image_urls.len() as i64)
                .unwrap_or(0),
            ..RequestUnits::default()
        };
        let body = if workload.streaming {
            vision_sample
                .map(|sample| responses::streaming_vision_response_body(model, sample, workload))
                .unwrap_or_else(|| responses::streaming_response_body(model, prompt, workload))
        } else {
            vision_sample
                .map(|sample| responses::vision_response_body(model, sample, workload))
                .unwrap_or_else(|| responses::response_body(model, prompt, workload))
        };
        let response = match self
            .with_protocol_auth(
                self.client.post(api_url(&config.base_url, "responses")),
                config,
                RealProviderProtocol::OpenAIResponses,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt.to_string()),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt.to_string()),
                None,
                None,
            );
        }
        if workload.streaming {
            return collect_streaming_response(
                response,
                RealProviderProtocol::OpenAIResponses,
                prompt,
                started,
                units,
            )
            .await;
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt.to_string()), None, None)
            }
        };
        let output = responses::extract_output_text(&payload);
        let usage =
            usage_from_value(&payload).unwrap_or_else(|| TokenUsage::estimated(prompt, &output));
        RequestOutcome::success_with_units(started.elapsed(), started.elapsed(), usage, units)
            .with_body(
                Some(prompt.to_string()),
                Some(output),
                raw_usage_from_value(&payload),
            )
    }

    async fn anthropic_message(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        prompt: &str,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
        vision_sample: Option<&VisionSample>,
    ) -> RequestOutcome {
        let started = Instant::now();
        let units = RequestUnits {
            image_count: vision_sample
                .map(|sample| sample.image_urls.len() as i64)
                .unwrap_or(0),
            ..RequestUnits::default()
        };
        let body = if workload.streaming {
            vision_sample
                .map(|sample| anthropic::streaming_vision_messages_body(model, sample, workload))
                .unwrap_or_else(|| anthropic::streaming_messages_body(model, prompt, workload))
        } else {
            vision_sample
                .map(|sample| anthropic::vision_messages_body(model, sample, workload))
                .unwrap_or_else(|| anthropic::messages_body(model, prompt, workload))
        };
        let response = match self
            .with_protocol_auth(
                self.client.post(api_url(&config.base_url, "messages")),
                config,
                RealProviderProtocol::Anthropic,
            )
            .header("anthropic-version", "2023-06-01")
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt.to_string()),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt.to_string()),
                None,
                None,
            );
        }
        if workload.streaming {
            return collect_streaming_response(
                response,
                RealProviderProtocol::Anthropic,
                prompt,
                started,
                units,
            )
            .await;
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt.to_string()), None, None)
            }
        };
        let output = anthropic::extract_text(&payload);
        let usage =
            usage_from_value(&payload).unwrap_or_else(|| TokenUsage::estimated(prompt, &output));
        RequestOutcome::success_with_units(started.elapsed(), started.elapsed(), usage, units)
            .with_body(
                Some(prompt.to_string()),
                Some(output),
                raw_usage_from_value(&payload),
            )
    }

    async fn gemini_generate_content(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        prompt: &str,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
        vision_sample: Option<&VisionSample>,
    ) -> RequestOutcome {
        let started = Instant::now();
        let units = RequestUnits {
            image_count: vision_sample
                .map(|sample| sample.image_urls.len() as i64)
                .unwrap_or(0),
            ..RequestUnits::default()
        };
        let body = vision_sample
            .map(|sample| gemini::vision_generate_content_body(sample, workload))
            .unwrap_or_else(|| gemini::generate_content_body(prompt, workload));
        let path = format!(
            "models/{}:{}",
            model.trim_start_matches("models/"),
            if workload.streaming {
                "streamGenerateContent"
            } else {
                "generateContent"
            }
        );
        let mut request = self.with_protocol_auth(
            self.client.post(api_url(&config.base_url, &path)),
            config,
            RealProviderProtocol::Gemini,
        );
        if workload.streaming {
            request = request.query(&[("alt", "sse")]);
        }
        let response = match request
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt.to_string()),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt.to_string()),
                None,
                None,
            );
        }
        if workload.streaming {
            return collect_streaming_response(
                response,
                RealProviderProtocol::Gemini,
                prompt,
                started,
                units,
            )
            .await;
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt.to_string()), None, None)
            }
        };
        let output = gemini::extract_text(&payload);
        let usage = gemini::usage_from_value(&payload)
            .map(|(input_tokens, output_tokens, total_tokens)| TokenUsage {
                input_tokens,
                output_tokens,
                total_tokens,
            })
            .unwrap_or_else(|| TokenUsage::estimated(prompt, &output));
        RequestOutcome::success_with_units(started.elapsed(), started.elapsed(), usage, units)
            .with_body(
                Some(prompt.to_string()),
                Some(output),
                payload.get("usageMetadata").cloned(),
            )
    }

    pub(super) async fn vision_completion(
        &self,
        config: &ProviderConnectionConfig,
        protocol: RealProviderProtocol,
        model: &str,
        sample: &DatasetSample,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        let started = Instant::now();
        let vision_sample = parse_vision_sample(&sample.prompt, workload.image_count);
        let prompt_for_log = sample.prompt.clone();
        match protocol {
            RealProviderProtocol::OpenAICompatible => {}
            RealProviderProtocol::OpenAIResponses => {
                return self
                    .openai_response(
                        config,
                        model,
                        &prompt_for_log,
                        workload,
                        request_timeout_seconds,
                        Some(&vision_sample),
                    )
                    .await;
            }
            RealProviderProtocol::Anthropic => {
                return self
                    .anthropic_message(
                        config,
                        model,
                        &prompt_for_log,
                        workload,
                        request_timeout_seconds,
                        Some(&vision_sample),
                    )
                    .await;
            }
            RealProviderProtocol::Gemini => {
                return self
                    .gemini_generate_content(
                        config,
                        model,
                        &prompt_for_log,
                        workload,
                        request_timeout_seconds,
                        Some(&vision_sample),
                    )
                    .await;
            }
        }
        let body = vision::vision_completion_body(model, &vision_sample, workload);
        let response = match self
            .with_auth(
                self.client
                    .post(api_url(&config.base_url, "chat/completions")),
                config,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt_for_log),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt_for_log),
                None,
                None,
            );
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt_for_log), None, None)
            }
        };
        let output = payload
            .pointer("/choices/0/message/content")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let usage = usage_from_value(&payload)
            .unwrap_or_else(|| TokenUsage::estimated(&sample.prompt, output));
        RequestOutcome::success_with_units(
            started.elapsed(),
            started.elapsed(),
            usage,
            RequestUnits {
                image_count: vision_sample.image_urls.len() as i64,
                ..RequestUnits::default()
            },
        )
        .with_body(
            Some(prompt_for_log),
            Some(output.to_string()),
            raw_usage_from_value(&payload),
        )
    }
}

impl RealProviderClient {
    fn with_auth(
        &self,
        request: reqwest::RequestBuilder,
        config: &ProviderConnectionConfig,
    ) -> reqwest::RequestBuilder {
        self.with_protocol_auth(request, config, RealProviderProtocol::OpenAICompatible)
    }

    pub(super) fn with_protocol_auth(
        &self,
        request: reqwest::RequestBuilder,
        config: &ProviderConnectionConfig,
        protocol: RealProviderProtocol,
    ) -> reqwest::RequestBuilder {
        let key = config.api_key_plaintext.trim();
        if key.is_empty() {
            request
        } else if protocol == RealProviderProtocol::Anthropic {
            request.header("x-api-key", key)
        } else if protocol == RealProviderProtocol::Gemini {
            request.query(&[("key", key)])
        } else {
            request.bearer_auth(key)
        }
    }
}
