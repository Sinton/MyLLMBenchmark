import { useState, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import { Copy } from "../../../components/ui/icons";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { LoadingBlock } from "../../../components/ui/LoadingBlock";
import { Tabs } from "../../../components/ui/Tabs";
import { formatDate } from "../domain/reportDefinitions";

type RequestDetailTab = "input" | "output" | "metrics";

type RequestLogExpandedRowProps = {
  requestId: string;
};

const detailTabs = [
  { key: "input", label: "输入" },
  { key: "output", label: "输出" },
  { key: "metrics", label: "指标" },
] as const;

export function RequestLogExpandedRow({ requestId }: RequestLogExpandedRowProps) {
  const [activeTab, setActiveTab] = useState<RequestDetailTab>("input");
  const detailQuery = useQuery({
    queryKey: queryKeys.benchmarkRequestLogDetail(requestId),
    queryFn: () => api.getBenchmarkRequestLogDetail(requestId),
  });

  if (detailQuery.isLoading) {
    return <LoadingBlock text="正在读取请求详情..." />;
  }

  if (detailQuery.isError || !detailQuery.data) {
    return (
      <div className="request-log-load-error">
        <InlineAlert tone="danger" title="请求详情读取失败">
          {detailQuery.error instanceof Error
            ? detailQuery.error.message
            : "无法读取该请求的持久化明细。"}
        </InlineAlert>
        <Button variant="ghost" onClick={() => detailQuery.refetch()}>
          重试
        </Button>
      </div>
    );
  }

  const detail = detailQuery.data;
  const prompt = detail.prompt ?? detail.prompt_preview;
  const response = detail.response_text ?? detail.response_preview;

  return (
    <div className="request-log-expanded">
      <Tabs
        ariaLabel="请求详情分类"
        items={detailTabs}
        value={activeTab}
        variant="line"
        onChange={setActiveTab}
      />

      {activeTab === "input" && (
        <div className="request-log-tab-panel" role="tabpanel">
          {!detail.body_available && (
            <InlineAlert title="仅有输入摘要" tone="warning">
              本次压测未保存 Prompt 正文，以下内容来自请求索引摘要。
            </InlineAlert>
          )}
          <RequestTextPanel
            emptyText="该请求没有可用的 Prompt 内容。"
            title="Prompt"
            value={prompt}
          />
        </div>
      )}

      {activeTab === "output" && (
        <div className="request-log-tab-panel" role="tabpanel">
          {!detail.body_available && (
            <InlineAlert title="仅有输出摘要" tone="warning">
              本次压测未保存响应正文，以下内容来自请求索引摘要。
            </InlineAlert>
          )}
          <RequestTextPanel
            emptyText={detail.status === "failed" ? "失败请求没有模型响应。" : "该请求没有可用的响应内容。"}
            title="模型响应"
            value={response}
          />
          {(detail.raw_error || detail.error_kind) && (
            <RequestTextPanel
              emptyText=""
              tone="danger"
              title="错误详情"
              value={detail.raw_error ?? detail.error_kind}
            />
          )}
        </div>
      )}

      {activeTab === "metrics" && (
        <div className="request-log-tab-panel" role="tabpanel">
          <div className="request-log-metrics-grid">
            <MetricItem label="阶段" value={`#${detail.stage_index}`} />
            <MetricItem label="请求序号" value={`#${detail.request_index}`} />
            <MetricItem label="样本序号" value={`#${detail.sample_index}`} />
            <MetricItem
              label="状态"
              value={
                <Badge tone={detail.status === "success" ? "success" : "danger"}>
                  {detail.status === "success" ? "成功" : "失败"}
                </Badge>
              }
            />
            <MetricItem label="总耗时" value={`${detail.latency_ms}ms`} />
            <MetricItem label="TTFT" value={detail.ttft_ms ? `${detail.ttft_ms}ms` : "-"} />
            <MetricItem label="输入 Token" value={detail.input_tokens.toLocaleString("zh-CN")} />
            <MetricItem label="输出 Token" value={detail.output_tokens.toLocaleString("zh-CN")} />
            <MetricItem label="总 Token" value={detail.total_tokens.toLocaleString("zh-CN")} />
            <MetricItem label="错误类型" value={detail.error_kind || "-"} />
            <MetricItem label="请求时间" value={formatDate(detail.created_at)} wide />
          </div>
          {detail.raw_usage != null && (
            <RequestTextPanel
              emptyText=""
              title="原始 Usage"
              value={formatStructuredValue(detail.raw_usage)}
            />
          )}
        </div>
      )}
    </div>
  );
}

function RequestTextPanel({
  title,
  value,
  emptyText,
  tone = "default",
}: {
  title: string;
  value?: string | null;
  emptyText: string;
  tone?: "default" | "danger";
}) {
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied" | "failed">("idle");

  const copy = async () => {
    if (!value) return;
    try {
      await navigator.clipboard.writeText(value);
      setCopyStatus("copied");
    } catch {
      setCopyStatus("failed");
    }
  };

  return (
    <section className={`request-log-text-panel request-log-text-${tone}`}>
      <header>
        <h4>{title}</h4>
        {value && (
          <Button icon={<Copy size={14} />} variant="ghost" onClick={copy}>
            {copyStatus === "copied"
              ? "已复制"
              : copyStatus === "failed"
                ? "复制失败"
                : "复制"}
          </Button>
        )}
      </header>
      {value ? <pre>{value}</pre> : <p>{emptyText}</p>}
    </section>
  );
}

function MetricItem({
  label,
  value,
  wide = false,
}: {
  label: string;
  value: ReactNode;
  wide?: boolean;
}) {
  return (
    <div className={wide ? "request-log-metric request-log-metric-wide" : "request-log-metric"}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function formatStructuredValue(value: unknown) {
  if (typeof value === "string") return value;
  return JSON.stringify(value, null, 2);
}
