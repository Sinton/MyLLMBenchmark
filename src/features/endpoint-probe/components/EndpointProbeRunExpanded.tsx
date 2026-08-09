import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Button } from "../../../components/ui/Button";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { LoadingBlock } from "../../../components/ui/LoadingBlock";
import { Tabs } from "../../../components/ui/Tabs";
import { ArrowRight, Building2, Copy } from "../../../components/ui/icons";
import type {
  EndpointProbeRunDetail,
  EndpointProbeRunSummary,
} from "../../../types/api";
import {
  canPromoteEndpointProbeRun,
  endpointProbeInterfaceLabel,
  endpointProbeStatusLabel,
} from "../domain/endpointProbePresentation";

type DetailTab = "response" | "request" | "metrics";

type EndpointProbeRunExpandedProps = {
  run: EndpointProbeRunSummary;
  detail?: EndpointProbeRunDetail;
  liveText: string;
  loading: boolean;
  error: string | null;
  onCopy: (label: string, value?: string | null) => Promise<void>;
  onRetry: () => void;
  onPromote: () => void;
};

export function EndpointProbeRunExpanded({
  run,
  detail,
  liveText,
  loading,
  error,
  onCopy,
  onRetry,
  onPromote,
}: EndpointProbeRunExpandedProps) {
  const [tab, setTab] = useState<DetailTab>("response");
  useEffect(() => setTab("response"), [run.id]);
  const running = run.status === "pending" || run.status === "running";
  const response = liveText || detail?.response_text || run.response_preview || "";
  const responseCopyValue = liveText || detail?.response_text || null;
  const promptCopyValue = detail?.prompt ?? null;
  const payloadCopyValue = detail?.request_payload
    ? formatJson(redactSensitiveValue(detail.request_payload))
    : null;
  const errorCopyValue = detail?.raw_error ?? run.error_message ?? null;
  const canPromote = canPromoteEndpointProbeRun(run);
  const failedReason = detail?.raw_error ?? run.error_message;

  return (
    <div className="endpoint-probe-run-expanded">
      <div className="endpoint-probe-run-expanded-head">
        <Tabs
          ariaLabel="请求详情"
          items={[
            { key: "response", label: "实时响应" },
            { key: "request", label: "请求" },
            { key: "metrics", label: "指标" },
          ]}
          value={tab}
          variant="line"
          onChange={setTab}
        />
        <div className="endpoint-probe-run-expanded-actions">
          {tab === "response" && (
            <CopyButton
              disabled={!responseCopyValue}
              label="复制响应"
              onClick={() => onCopy("响应", responseCopyValue)}
            />
          )}
          {tab === "request" && (
            <>
              <CopyButton
                disabled={!promptCopyValue}
                label="复制 Prompt"
                onClick={() => onCopy("Prompt", promptCopyValue)}
              />
              <CopyButton
                disabled={!payloadCopyValue}
                label="复制 Payload"
                onClick={() => onCopy("Payload", payloadCopyValue)}
              />
            </>
          )}
          {canPromote && (
            <Button icon={<Building2 size={15} />} variant="primary" onClick={onPromote}>
              保存为服务商
            </Button>
          )}
        </div>
      </div>

      {run.status === "failed" && failedReason && (
        <InlineAlert tone="danger" title={run.error_kind ?? "请求失败"}>
          <div className="endpoint-probe-error-detail">
            <span>{failedReason}</span>
            <CopyButton
              disabled={!errorCopyValue}
              label="复制错误"
              onClick={() => onCopy("错误", errorCopyValue)}
            />
          </div>
        </InlineAlert>
      )}

      {error ? (
        <InlineAlert tone="danger" title="请求详情读取失败">
          {error}
          <Button variant="ghost" onClick={onRetry}>重试</Button>
        </InlineAlert>
      ) : loading ? (
        <LoadingBlock text="正在读取请求详情..." />
      ) : tab === "response" ? (
        <StreamingResponse running={running} text={response} />
      ) : tab === "request" ? (
        <div className="endpoint-probe-code-stack">
          {!detail?.body_available && !running && (
            <InlineAlert tone="info" title="历史正文未保存">
              当前仅有请求与响应摘要；完整正文只能在测活前显式开启保存。
            </InlineAlert>
          )}
          <CodeBlock
            title="Prompt"
            value={detail?.prompt ?? run.prompt_preview ?? "请求执行完成后显示 Prompt 摘要"}
          />
          <CodeBlock title="请求 Payload" value={payloadCopyValue ?? "无"} />
        </div>
      ) : (
        <div className="endpoint-probe-run-metrics">
          <Fact label="状态" value={endpointProbeStatusLabel(run.status)} />
          <Fact label="接口" value={endpointProbeInterfaceLabel(run.interface_type)} />
          <Fact label="模型" value={run.model} />
          <Fact label="TTFT" value={formatMs(run.ttft_ms)} />
          <Fact label="总耗时" value={formatMs(run.latency_ms)} />
          <Fact label="输入 Token" value={String(run.input_tokens)} />
          <Fact label="输出 Token" value={String(run.output_tokens)} />
          <Fact label="总 Token" value={String(run.total_tokens)} />
          <Fact label="错误类型" value={run.error_kind ?? "-"} />
          <CodeBlock title="Raw Usage" value={formatJson(detail?.raw_usage)} />
        </div>
      )}
    </div>
  );
}

function CopyButton({
  disabled,
  label,
  onClick,
}: {
  disabled: boolean;
  label: string;
  onClick: () => void;
}) {
  return (
    <Button
      disabled={disabled}
      icon={<Copy size={14} />}
      title={disabled ? "仅保存摘要，暂无完整正文可复制" : label}
      variant="ghost"
      onClick={onClick}
    >
      {label}
    </Button>
  );
}

function StreamingResponse({ text, running }: { text: string; running: boolean }) {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const [followOutput, setFollowOutput] = useState(true);

  useLayoutEffect(() => {
    if (!followOutput || !viewportRef.current) return;
    viewportRef.current.scrollTop = viewportRef.current.scrollHeight;
  }, [followOutput, text]);

  return (
    <div className="endpoint-probe-stream-shell">
      <div className="endpoint-probe-stream-toolbar">
        <span className={running ? "is-live" : ""}>
          {running ? "SSE 接收中" : "响应已完成"}
        </span>
        {!followOutput && (
          <Button
            icon={<ArrowRight size={14} />}
            variant="ghost"
            onClick={() => {
              setFollowOutput(true);
              if (viewportRef.current) {
                viewportRef.current.scrollTop = viewportRef.current.scrollHeight;
              }
            }}
          >
            回到底部
          </Button>
        )}
      </div>
      <div
        className="endpoint-probe-stream-output"
        ref={viewportRef}
        onScroll={(event) => {
          const target = event.currentTarget;
          setFollowOutput(target.scrollHeight - target.scrollTop - target.clientHeight < 28);
        }}
      >
        {text ? <pre>{text}</pre> : <span>{running ? "等待首个响应 chunk..." : "无响应正文"}</span>}
      </div>
    </div>
  );
}

function CodeBlock({ title, value }: { title: string; value: string }) {
  return (
    <section className="endpoint-probe-code-block">
      <strong>{title}</strong>
      <pre>{value}</pre>
    </section>
  );
}

function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div className="endpoint-probe-run-fact">
      <span>{label}</span>
      <strong title={value}>{value}</strong>
    </div>
  );
}

function formatJson(value: unknown) {
  if (value === null || value === undefined) return "无";
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function redactSensitiveValue(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(redactSensitiveValue);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>).map(([key, item]) => [
      key,
      isSensitiveKey(key) ? "[REDACTED]" : redactSensitiveValue(item),
    ]),
  );
}

function isSensitiveKey(key: string) {
  const normalized = key.toLowerCase();
  return [
    "api_key",
    "apikey",
    "authorization",
    "cookie",
    "x-api-key",
    "anthropic-api-key",
    "openai-api-key",
  ].includes(normalized) || normalized.includes("secret");
}

function formatMs(value: number) {
  return value > 0 ? `${value.toLocaleString("zh-CN")} ms` : "-";
}
