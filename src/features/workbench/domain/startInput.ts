import type { BenchmarkStartInput } from "../../../types/api";
import {
  finiteNumber,
  firstError,
  greaterThan,
  invalid,
  numberRange,
} from "../../../lib/validation";
import { buildWorkloadConfig } from "./workloadConfig";
import type { WorkbenchForm } from "../types";

export type StartInputResult =
  | { ok: true; input: BenchmarkStartInput }
  | { ok: false; message: string };

type BuildStartInputOptions = {
  estimatedSeconds: number;
  form: WorkbenchForm;
  isStaircase: boolean;
  selectedModelType: string;
};

export function buildBenchmarkStartInput({
  estimatedSeconds,
  form,
  isStaircase,
  selectedModelType,
}: BuildStartInputOptions): StartInputResult {
  const numericError = validateNumericFields(form, estimatedSeconds);
  if (numericError) {
    return { ok: false, message: numericError };
  }

  return {
    ok: true,
    input: {
      provider_id: form.provider_id,
      model_id: form.model_id || null,
      dataset_id: form.dataset_id,
      mode: form.mode,
      concurrency: isStaircase ? form.end_concurrency : form.concurrency,
      duration_seconds: estimatedSeconds,
      start_concurrency: form.start_concurrency,
      end_concurrency: form.end_concurrency,
      step_strategy: form.step_strategy as "double" | "linear",
      step_value: form.step_value,
      stage_sample_rounds: form.stage_sample_rounds,
      stage_duration_seconds: form.stage_sample_rounds,
      warmup_rounds: form.warmup_rounds,
      warmup_seconds: form.warmup_rounds,
      request_timeout_seconds: form.request_timeout_seconds,
      sla_p95_ms: form.sla_p95_ms,
      min_success_rate: form.min_success_rate,
      sla_stop_policy: form.sla_stop_policy,
      workload_config: buildWorkloadConfig(selectedModelType, form),
      request_log_config: {
        enabled: form.request_log_enabled,
        capture_body: form.request_log_enabled && form.request_log_capture_body,
        max_records_per_stage: form.request_log_max_records_per_stage,
      },
    },
  };
}

function validateNumericFields(form: WorkbenchForm, estimatedSeconds: number) {
  const numericError = firstError([
    finiteNumber("并发", form.concurrency),
    finiteNumber("固定模式请求轮次", form.duration_seconds),
    finiteNumber("起始并发", form.start_concurrency),
    finiteNumber("结束并发", form.end_concurrency),
    finiteNumber("步长", form.step_value),
    finiteNumber("每阶段请求轮次", form.stage_sample_rounds),
    finiteNumber("预热轮次", form.warmup_rounds),
    finiteNumber("请求超时", form.request_timeout_seconds),
    finiteNumber("P95 SLA", form.sla_p95_ms),
    finiteNumber("最低成功率", form.min_success_rate),
    finiteNumber("请求明细保存上限", form.request_log_max_records_per_stage),
    finiteNumber("预计请求轮次", estimatedSeconds),
  ]);
  if (numericError) return numericError;

  if (form.concurrency < 1) return "并发必须大于 0。";
  if (form.start_concurrency < 1) return "起始并发必须大于 0。";
  if (form.end_concurrency < form.start_concurrency) {
    return "结束并发不能小于起始并发。";
  }
  return firstError([
    greaterThan("阶梯步长", form.step_value, 0),
    greaterThan("每阶段请求轮次", form.stage_sample_rounds, 0),
    form.warmup_rounds >= 0 ? { ok: true } : invalid("预热轮次不能小于 0。"),
    numberRange("请求超时", form.request_timeout_seconds, 5, 600),
    greaterThan("P95 SLA", form.sla_p95_ms, 0),
    numberRange("最低成功率", form.min_success_rate, 0, 100),
    numberRange("请求明细保存上限", form.request_log_max_records_per_stage, 1, 1000),
  ]);
}
