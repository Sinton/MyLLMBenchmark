use super::MockDataStore;
use crate::domain::dataset_import::{estimate_tokens, parse_dataset_import};
use crate::domain::dataset_tools::{
    dataset_export_result, render_dataset_export, validate_dataset_samples,
};
use crate::error::AppError;
use crate::models::{
    DatasetAppendInput, DatasetExportInput, DatasetExportResult, DatasetImportInput, DatasetSample,
    DatasetSampleBatchDeleteInput, DatasetSampleCreateInput, DatasetSamplePage,
    DatasetSamplePageInput, DatasetSamplePreview, DatasetSampleUpdateInput, DatasetSummary,
    DatasetUpdateInput, DatasetValidationResult, DeleteResult,
};
use uuid::Uuid;

impl MockDataStore {
    pub async fn list_datasets(&self) -> anyhow::Result<Vec<DatasetSummary>> {
        let data = self.inner.read().await;
        Ok(data.datasets.iter().rev().cloned().collect())
    }

    pub async fn import_dataset(
        &self,
        input: DatasetImportInput,
    ) -> anyhow::Result<DatasetSummary> {
        let parsed = parse_dataset_import(input)?;
        let mut data = self.inner.write().await;
        let id = Uuid::new_v4().to_string();
        let now = super::now();
        let dataset = DatasetSummary {
            id: id.clone(),
            name: parsed.name,
            dataset_type: parsed.dataset_type,
            sample_count: parsed.prompts.len() as i64,
            average_tokens: parsed.average_tokens,
            updated_at: now,
        };
        let samples = parsed
            .prompts
            .into_iter()
            .enumerate()
            .map(|(index, prompt)| DatasetSample {
                id: Uuid::new_v4().to_string(),
                dataset_id: id.clone(),
                sample_index: index as i64,
                estimated_tokens: estimate_tokens(&prompt),
                prompt,
            })
            .collect();
        data.dataset_samples.insert(id.clone(), samples);
        data.datasets.push(dataset.clone());
        Ok(dataset)
    }

    pub async fn update_dataset(
        &self,
        input: DatasetUpdateInput,
    ) -> anyhow::Result<DatasetSummary> {
        let name = normalize_required(&input.name, "数据集名称不能为空")?;
        let dataset_type = normalize_required(&input.dataset_type, "数据集类型不能为空")?;
        let mut data = self.inner.write().await;
        let dataset = data
            .datasets
            .iter_mut()
            .find(|dataset| dataset.id == input.id)
            .ok_or_else(|| AppError::not_found("dataset"))?;
        dataset.name = name;
        dataset.dataset_type = dataset_type;
        dataset.updated_at = super::now();
        Ok(dataset.clone())
    }

    pub async fn delete_dataset(&self, dataset_id: &str) -> anyhow::Result<DeleteResult> {
        let mut data = self.inner.write().await;
        let before = data.datasets.len();
        data.datasets.retain(|dataset| dataset.id != dataset_id);
        let deleted = data.datasets.len() != before;
        if deleted {
            data.dataset_samples.remove(dataset_id);
        }
        Ok(DeleteResult {
            id: dataset_id.to_string(),
            deleted,
        })
    }

    pub async fn preview_dataset_samples(
        &self,
        dataset_id: &str,
        limit: i64,
    ) -> anyhow::Result<Vec<DatasetSamplePreview>> {
        let data = self.inner.read().await;
        ensure_dataset_exists(&data.datasets, dataset_id)?;
        let samples = data
            .dataset_samples
            .get(dataset_id)
            .cloned()
            .unwrap_or_default();
        let visible_samples = if limit <= 0 {
            samples.iter().collect::<Vec<_>>()
        } else {
            samples
                .iter()
                .take(limit.clamp(1, 10_000) as usize)
                .collect::<Vec<_>>()
        };
        Ok(visible_samples.into_iter().map(sample_preview).collect())
    }

    pub async fn list_dataset_samples_page(
        &self,
        input: DatasetSamplePageInput,
    ) -> anyhow::Result<DatasetSamplePage> {
        let data = self.inner.read().await;
        ensure_dataset_exists(&data.datasets, &input.dataset_id)?;
        let page = input.page.max(1);
        let page_size = normalize_page_size(input.page_size)?;
        let keyword = normalize_keyword(input.keyword).map(|value| value.to_lowercase());
        let samples = data
            .dataset_samples
            .get(&input.dataset_id)
            .cloned()
            .unwrap_or_default();
        let filtered = samples
            .iter()
            .filter(|sample| {
                keyword
                    .as_ref()
                    .map(|value| sample.prompt.to_lowercase().contains(value))
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        let total = filtered.len() as i64;
        let offset = ((page - 1) * page_size).max(0) as usize;
        let items = filtered
            .into_iter()
            .skip(offset)
            .take(page_size as usize)
            .map(sample_preview)
            .collect();

        Ok(DatasetSamplePage {
            items,
            total,
            page,
            page_size,
        })
    }

    pub async fn list_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<Vec<DatasetSample>> {
        let data = self.inner.read().await;
        ensure_dataset_exists(&data.datasets, dataset_id)?;
        Ok(data
            .dataset_samples
            .get(dataset_id)
            .cloned()
            .unwrap_or_default())
    }

    pub async fn create_dataset_sample(
        &self,
        input: DatasetSampleCreateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        let prompt = normalize_prompt(&input.prompt)?;
        let mut data = self.inner.write().await;
        ensure_dataset_exists(&data.datasets, &input.dataset_id)?;
        let sample = DatasetSample {
            id: Uuid::new_v4().to_string(),
            dataset_id: input.dataset_id.clone(),
            sample_index: data
                .dataset_samples
                .get(&input.dataset_id)
                .and_then(|samples| samples.iter().map(|sample| sample.sample_index).max())
                .map(|max| max + 1)
                .unwrap_or(0),
            estimated_tokens: estimate_tokens(&prompt),
            prompt,
        };
        data.dataset_samples
            .entry(input.dataset_id.clone())
            .or_default()
            .push(sample.clone());
        let (sample_count, average_tokens) =
            stats_for_samples(data.dataset_samples.get(&input.dataset_id));
        recompute_dataset_stats(
            &mut data.datasets,
            &input.dataset_id,
            sample_count,
            average_tokens,
        )?;
        Ok(sample_preview(&sample))
    }

    pub async fn update_dataset_sample(
        &self,
        input: DatasetSampleUpdateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        let prompt = normalize_prompt(&input.prompt)?;
        let mut data = self.inner.write().await;
        let dataset_id = find_dataset_id_for_sample(&data.dataset_samples, &input.sample_id)?;
        ensure_dataset_exists(&data.datasets, &dataset_id)?;
        let samples = data
            .dataset_samples
            .get_mut(&dataset_id)
            .ok_or_else(|| AppError::not_found("dataset samples"))?;
        let sample = samples
            .iter_mut()
            .find(|sample| sample.id == input.sample_id)
            .ok_or_else(|| AppError::not_found("dataset sample"))?;
        sample.prompt = prompt;
        sample.estimated_tokens = estimate_tokens(&sample.prompt);
        let updated = sample.clone();
        let (sample_count, average_tokens) =
            stats_for_samples(data.dataset_samples.get(&dataset_id));
        recompute_dataset_stats(
            &mut data.datasets,
            &dataset_id,
            sample_count,
            average_tokens,
        )?;
        Ok(sample_preview(&updated))
    }

    pub async fn delete_dataset_sample(&self, sample_id: &str) -> anyhow::Result<DeleteResult> {
        let mut data = self.inner.write().await;
        let dataset_id = find_dataset_id_for_sample(&data.dataset_samples, sample_id)?;
        ensure_dataset_exists(&data.datasets, &dataset_id)?;
        let samples = data
            .dataset_samples
            .get_mut(&dataset_id)
            .ok_or_else(|| AppError::not_found("dataset samples"))?;
        let before = samples.len();
        samples.retain(|sample| sample.id != sample_id);
        let deleted = samples.len() != before;
        if deleted {
            for (index, sample) in samples.iter_mut().enumerate() {
                sample.sample_index = index as i64;
            }
            let (sample_count, average_tokens) =
                stats_for_samples(data.dataset_samples.get(&dataset_id));
            recompute_dataset_stats(
                &mut data.datasets,
                &dataset_id,
                sample_count,
                average_tokens,
            )?;
        }
        Ok(DeleteResult {
            id: sample_id.to_string(),
            deleted,
        })
    }

    pub async fn append_dataset_samples(
        &self,
        input: DatasetAppendInput,
    ) -> anyhow::Result<DatasetSummary> {
        let mut data = self.inner.write().await;
        let dataset = data
            .datasets
            .iter()
            .find(|dataset| dataset.id == input.dataset_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("dataset"))?;
        let parsed = parse_dataset_import(DatasetImportInput {
            name: dataset.name,
            dataset_type: dataset.dataset_type,
            format: input.format,
            file_name: input.file_name,
            content_base64: input.content_base64,
        })?;
        let samples = data
            .dataset_samples
            .entry(input.dataset_id.clone())
            .or_default();
        if samples.len() + parsed.prompts.len() > 10_000 {
            return Err(AppError::validation("数据集最多保留 10000 条样本").into());
        }
        let mut next_index = samples
            .iter()
            .map(|sample| sample.sample_index)
            .max()
            .map(|index| index + 1)
            .unwrap_or(0);
        for prompt in parsed.prompts {
            samples.push(DatasetSample {
                id: Uuid::new_v4().to_string(),
                dataset_id: input.dataset_id.clone(),
                sample_index: next_index,
                estimated_tokens: estimate_tokens(&prompt),
                prompt,
            });
            next_index += 1;
        }
        let (sample_count, average_tokens) =
            stats_for_samples(data.dataset_samples.get(&input.dataset_id));
        recompute_dataset_stats(
            &mut data.datasets,
            &input.dataset_id,
            sample_count,
            average_tokens,
        )?;
        data.datasets
            .iter()
            .find(|dataset| dataset.id == input.dataset_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("dataset").into())
    }

    pub async fn delete_dataset_samples_batch(
        &self,
        input: DatasetSampleBatchDeleteInput,
    ) -> anyhow::Result<DeleteResult> {
        let mut data = self.inner.write().await;
        ensure_dataset_exists(&data.datasets, &input.dataset_id)?;
        let ids = input
            .sample_ids
            .into_iter()
            .collect::<std::collections::HashSet<_>>();
        if ids.is_empty() {
            return Ok(DeleteResult {
                id: input.dataset_id,
                deleted: false,
            });
        }
        let samples = data
            .dataset_samples
            .get_mut(&input.dataset_id)
            .ok_or_else(|| AppError::not_found("dataset samples"))?;
        let before = samples.len();
        samples.retain(|sample| !ids.contains(&sample.id));
        let deleted = samples.len() != before;
        if deleted {
            for (index, sample) in samples.iter_mut().enumerate() {
                sample.sample_index = index as i64;
            }
            let (sample_count, average_tokens) =
                stats_for_samples(data.dataset_samples.get(&input.dataset_id));
            recompute_dataset_stats(
                &mut data.datasets,
                &input.dataset_id,
                sample_count,
                average_tokens,
            )?;
        }
        Ok(DeleteResult {
            id: input.dataset_id,
            deleted,
        })
    }

    pub async fn export_dataset(
        &self,
        input: DatasetExportInput,
    ) -> anyhow::Result<DatasetExportResult> {
        let data = self.inner.read().await;
        let dataset = data
            .datasets
            .iter()
            .find(|dataset| dataset.id == input.dataset_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("dataset"))?;
        let samples = data
            .dataset_samples
            .get(&input.dataset_id)
            .cloned()
            .unwrap_or_default();
        let payload = render_dataset_export(&samples, &input.format);
        Ok(dataset_export_result(
            &dataset,
            &payload,
            String::new(),
            String::new(),
        ))
    }

    pub async fn validate_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<DatasetValidationResult> {
        let data = self.inner.read().await;
        let dataset = data
            .datasets
            .iter()
            .find(|dataset| dataset.id == dataset_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("dataset"))?;
        let samples = data
            .dataset_samples
            .get(dataset_id)
            .cloned()
            .unwrap_or_default();
        Ok(validate_dataset_samples(
            dataset_id,
            &dataset.dataset_type,
            &samples,
        ))
    }
}

fn ensure_dataset_exists(datasets: &[DatasetSummary], dataset_id: &str) -> anyhow::Result<()> {
    if datasets.iter().any(|dataset| dataset.id == dataset_id) {
        return Ok(());
    }
    Err(AppError::not_found("dataset").into())
}

fn find_dataset_id_for_sample(
    samples_by_dataset: &std::collections::HashMap<String, Vec<DatasetSample>>,
    sample_id: &str,
) -> anyhow::Result<String> {
    samples_by_dataset
        .iter()
        .find_map(|(dataset_id, samples)| {
            samples
                .iter()
                .any(|sample| sample.id == sample_id)
                .then(|| dataset_id.clone())
        })
        .ok_or_else(|| AppError::not_found("dataset sample").into())
}

fn recompute_dataset_stats(
    datasets: &mut [DatasetSummary],
    dataset_id: &str,
    sample_count: i64,
    average_tokens: i64,
) -> anyhow::Result<()> {
    let dataset = datasets
        .iter_mut()
        .find(|dataset| dataset.id == dataset_id)
        .ok_or_else(|| AppError::not_found("dataset"))?;
    dataset.sample_count = sample_count;
    dataset.average_tokens = average_tokens;
    dataset.updated_at = super::now();
    Ok(())
}

fn stats_for_samples(samples: Option<&Vec<DatasetSample>>) -> (i64, i64) {
    let Some(samples) = samples else {
        return (0, 0);
    };
    if samples.is_empty() {
        return (0, 0);
    }
    (
        samples.len() as i64,
        samples
            .iter()
            .map(|sample| sample.estimated_tokens)
            .sum::<i64>()
            / samples.len() as i64,
    )
}

fn sample_preview(sample: &DatasetSample) -> DatasetSamplePreview {
    DatasetSamplePreview {
        id: sample.id.clone(),
        sample_index: sample.sample_index,
        prompt_preview: preview_prompt(&sample.prompt),
        prompt: sample.prompt.clone(),
        estimated_tokens: sample.estimated_tokens,
    }
}

fn normalize_required(value: &str, message: &str) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::validation(message).into());
    }
    Ok(value.to_string())
}

fn normalize_prompt(prompt: &str) -> anyhow::Result<String> {
    normalize_required(prompt, "Prompt 样本不能为空")
}

fn normalize_keyword(keyword: Option<String>) -> Option<String> {
    keyword
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_page_size(page_size: i64) -> anyhow::Result<i64> {
    match page_size {
        0 => Ok(50),
        20 | 50 | 100 | 200 => Ok(page_size),
        value if value > 200 => Ok(200),
        _ => Err(AppError::validation("page_size 只支持 20、50、100、200").into()),
    }
}

fn preview_prompt(prompt: &str) -> String {
    const MAX_CHARS: usize = 120;
    let mut preview = prompt.chars().take(MAX_CHARS).collect::<String>();
    if prompt.chars().count() > MAX_CHARS {
        preview.push_str("...");
    }
    preview
}
