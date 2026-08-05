export type DataMode = "mock" | "sqlite";

export type BenchmarkEngineMode = "mock" | "openai_compatible";

export type NotificationPosition =
  | "top-right"
  | "top-left"
  | "bottom-right"
  | "bottom-left";

export type EndpointProbePromptTemplate = {
  id: string;
  name: string;
  prompt: string;
};

export type EndpointProbePromptTemplatesConfig = {
  selected_id: string;
  items: EndpointProbePromptTemplate[];
};

export type AppConfig = {
  data_mode: DataMode;
  benchmark_engine: BenchmarkEngineMode;
  notification_position: NotificationPosition;
  endpoint_probe_prompt_templates: EndpointProbePromptTemplatesConfig;
};

export type ConfigUpdateResult = {
  config: AppConfig;
  restart_required: boolean;
};
