export type DataMode = "mock" | "sqlite";

export type BenchmarkEngineMode = "mock" | "openai_compatible";

export type AppConfig = {
  data_mode: DataMode;
  benchmark_engine: BenchmarkEngineMode;
};

export type ConfigUpdateResult = {
  config: AppConfig;
  restart_required: boolean;
};
