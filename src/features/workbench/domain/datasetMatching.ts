import { normalizeModelType } from "../../../lib/modelTaxonomy";

export function datasetTypeForModel(modelTypeValue: string) {
  const modelType = normalizeModelType(modelTypeValue);
  if (modelType === "embedding") return "embedding";
  if (modelType === "rerank") return "rerank";
  if (modelType === "multimodal") return "multimodal";
  return "text_generation";
}

export function normalizeDatasetType(value: string) {
  return normalizeModelType(value);
}
