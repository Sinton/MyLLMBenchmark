import { normalizeModelType } from "../../../lib/modelTaxonomy";
import type { BenchmarkWorkloadConfig } from "../../../types/api";
import type { WorkbenchForm } from "../types";

export function buildWorkloadConfig(
  modelTypeValue: string,
  form: WorkbenchForm,
): BenchmarkWorkloadConfig {
  const modelType = normalizeModelType(modelTypeValue);
  const base = defaultWorkloadConfig(modelType);

  if (modelType === "embedding") {
    return {
      ...base,
      batch_size: Number(form.embedding_batch_size),
      text_count_per_request: Number(form.embedding_text_count_per_request),
    };
  }

  if (modelType === "rerank") {
    return {
      ...base,
      documents_per_query: Number(form.rerank_documents_per_query),
      top_k: Number(form.rerank_top_k),
    };
  }

  if (modelType === "multimodal") {
    return {
      ...base,
      image_profile: form.vision_image_profile as BenchmarkWorkloadConfig["image_profile"],
      image_count: Number(form.vision_image_count),
    };
  }

  return {
    ...base,
    streaming: form.streaming,
    max_output_tokens: Number(form.max_output_tokens),
    prompt_profile: form.prompt_profile as BenchmarkWorkloadConfig["prompt_profile"],
  };
}

function defaultWorkloadConfig(modelType: string): BenchmarkWorkloadConfig {
  if (modelType === "embedding") {
    return { batch_size: 16, text_count_per_request: 16 };
  }
  if (modelType === "rerank") {
    return { documents_per_query: 30, top_k: 10 };
  }
  if (modelType === "multimodal") {
    return {
      image_profile: "medium",
      image_count: 1,
      max_output_tokens: 512,
      streaming: true,
    };
  }
  return { streaming: true, max_output_tokens: 512, prompt_profile: "mixed" };
}
