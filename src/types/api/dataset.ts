export type DatasetSummary = {
  id: string;
  name: string;
  dataset_type: string;
  sample_count: number;
  average_tokens: number;
  updated_at: string;
};

export type DatasetImportInput = {
  name: string;
  dataset_type: string;
  format: string;
  file_name: string;
  content_base64: string;
};

export type DatasetAppendInput = {
  dataset_id: string;
  format: string;
  file_name: string;
  content_base64: string;
};

export type DatasetUpdateInput = {
  id: string;
  name: string;
  dataset_type: string;
};

export type DatasetSamplePreview = {
  id: string;
  sample_index: number;
  prompt: string;
  prompt_preview: string;
  estimated_tokens: number;
};

export type DatasetSamplePageInput = {
  dataset_id: string;
  page: number;
  page_size: number;
  keyword?: string | null;
};

export type DatasetSamplePage = {
  items: DatasetSamplePreview[];
  total: number;
  page: number;
  page_size: number;
};

export type DatasetSampleCreateInput = {
  dataset_id: string;
  prompt: string;
};

export type DatasetSampleUpdateInput = {
  sample_id: string;
  prompt: string;
};

export type DatasetSampleBatchDeleteInput = {
  dataset_id: string;
  sample_ids: string[];
};

export type DatasetExportInput = {
  dataset_id: string;
  format: "jsonl" | "csv" | "txt" | string;
};

export type DatasetExportResult = {
  dataset_id: string;
  format: string;
  file_name: string;
  file_path: string;
  mime_type: string;
  sample_count: number;
  message: string;
};

export type DatasetValidationIssue = {
  kind: string;
  label: string;
  count: number;
  sample_indexes: number[];
};

export type DatasetValidationResult = {
  dataset_id: string;
  status: "passed" | "warning" | "failed" | string;
  checked_at: string;
  sample_count: number;
  issues: DatasetValidationIssue[];
  recommendations: string[];
};
