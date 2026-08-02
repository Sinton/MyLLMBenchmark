import { useEffect, useState } from "react";
import { Badge } from "../../../components/ui/Badge";
import { Card } from "../../../components/ui/Card";
import { EmptyState } from "../../../components/ui/EmptyState";
import { Gauge, Network } from "../../../components/ui/icons";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { LoadingBlock } from "../../../components/ui/LoadingBlock";
import { MetricCard } from "../../../components/ui/MetricCard";
import { Tabs } from "../../../components/ui/Tabs";
import type { SiteProbeRunDetail } from "../../../types/api";
import { siteProbeInterfaceLabel } from "../domain/siteProbePresentation";

type ResultTab = "response" | "request" | "metrics";

type SiteProbeResultProps = {
  detail: SiteProbeRunDetail | null;
  loading: boolean;
  error: unknown;
};

const resultTabs = [
  { key: "response", label: "响应" },
  { key: "request", label: "请求" },
  { key: "metrics", label: "指标" },
] as const;

export function SiteProbeResult({
  detail,
  loading,
  error,
}: SiteProbeResultProps) {
  const [tab, setTab] = useState<ResultTab>("response");

  useEffect(() => {
    setTab("response");
  }, [detail?.id]);

  if (loading) {
    return (
      <Card className="site-probe-result-card">
        <LoadingBlock text="正在读取测活详情..." />
      </Card>
    );
  }

  if (error) {
    return (
      <Card className="site-probe-result-card">
        <InlineAlert tone="danger" title="详情读取失败">
          {error instanceof Error ? error.message : String(error)}
        </InlineAlert>
      </Card>
    );
  }

  if (!detail) {
    return (
      <Card className="site-probe-result-card">
        <EmptyState
          icon={<Network size={22} />}
          title="还没有测活结果"
          description="填写中转站、模型和 Prompt 后发起一次请求，这里会显示响应正文、TTFT、Token 用量和错误详情。"
        />
      </Card>
    );
  }

  const bodyMissing = !detail.body_available && !detail.prompt && !detail.response_text;

  return (
    <Card className="site-probe-result-card">
      <div className="site-probe-result-head">
        <div>
          <h2>{detail.name}</h2>
          <p>
            {detail.base_url} · {detail.model}
          </p>
        </div>
        <Badge tone={detail.status === "passed" ? "success" : "danger"}>
          {detail.status === "passed" ? "可用" : "失败"}
        </Badge>
      </div>

      <div className="site-probe-metrics">
        <MetricCard label="总耗时" value={detail.latency_ms} unit="ms" />
        <MetricCard label="TTFT" value={detail.ttft_ms || "-"} unit={detail.ttft_ms ? "ms" : undefined} />
        <MetricCard label="输入 Token" value={detail.input_tokens} />
        <MetricCard label="输出 Token" value={detail.output_tokens} />
      </div>

      {bodyMissing && (
        <InlineAlert tone="info" title="该历史记录未保存正文">
          这里只能展示 Prompt / 响应摘要。需要查看完整输入输出，请在测活前开启“保存正文”。
        </InlineAlert>
      )}

      {detail.status !== "passed" && detail.error_message && (
        <InlineAlert tone="danger" title={detail.error_kind ?? "请求失败"}>
          {detail.error_message}
        </InlineAlert>
      )}

      <div className="site-probe-result-tabs">
        <Tabs
          ariaLabel="测活结果详情"
          items={resultTabs}
          value={tab}
          variant="line"
          onChange={setTab}
        />
        {tab === "response" && (
          <div className="site-probe-code-grid">
            <CodeBlock
              title="模型响应"
              value={detail.response_text ?? detail.response_preview ?? "无响应正文"}
            />
            {detail.raw_error && (
              <CodeBlock title="错误详情" value={detail.raw_error} tone="danger" />
            )}
          </div>
        )}
        {tab === "request" && (
          <div className="site-probe-code-grid two">
            <CodeBlock
              title="Prompt"
              value={detail.prompt ?? detail.prompt_preview ?? "无 Prompt 正文"}
            />
            <CodeBlock
              title="请求 Payload"
              value={formatJson(detail.request_payload)}
            />
          </div>
        )}
        {tab === "metrics" && (
          <div className="site-probe-facts">
            <Fact label="请求状态" value={detail.status} />
            <Fact label="接口类型" value={siteProbeInterfaceLabel(detail.interface_type)} />
            <Fact label="模型" value={detail.model} />
            <Fact label="创建时间" value={formatDate(detail.created_at)} />
            <Fact label="总 Token" value={String(detail.total_tokens)} />
            <Fact label="错误类型" value={detail.error_kind ?? "-"} />
            <div className="site-probe-usage">
              <div className="site-probe-usage-title">
                <Gauge size={15} />
                <strong>Raw Usage</strong>
              </div>
              <pre>{formatJson(detail.raw_usage)}</pre>
            </div>
          </div>
        )}
      </div>
    </Card>
  );
}

function CodeBlock({
  title,
  value,
  tone,
}: {
  title: string;
  value: string;
  tone?: "danger";
}) {
  return (
    <section className={`site-probe-code ${tone ? `is-${tone}` : ""}`}>
      <strong>{title}</strong>
      <pre>{value}</pre>
    </section>
  );
}

function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div className="site-probe-fact">
      <span>{label}</span>
      <strong>{value}</strong>
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

function formatDate(value: string) {
  return new Date(value).toLocaleString("zh-CN");
}
