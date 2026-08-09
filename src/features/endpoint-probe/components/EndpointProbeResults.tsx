import { MetricHelp } from "../../../components/common/MetricHelp";
import { Badge } from "../../../components/ui/Badge";
import { Card } from "../../../components/ui/Card";
import { DataTable, type DataTableColumn } from "../../../components/ui/DataTable";
import { EmptyState } from "../../../components/ui/EmptyState";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { LoadingBlock } from "../../../components/ui/LoadingBlock";
import { MetricCard } from "../../../components/ui/MetricCard";
import { Button } from "../../../components/ui/Button";
import { Building2, Network } from "../../../components/ui/icons";
import type { EndpointProbeRunSummary } from "../../../types/api";
import {
  canPromoteEndpointProbeRun,
  endpointProbeInterfaceLabel,
  endpointProbeRunResultText,
  endpointProbeStatusLabel,
  endpointProbeStatusTone,
} from "../domain/endpointProbePresentation";
import type { useEndpointProbeView } from "../hooks/useEndpointProbeView";
import { EndpointProbeRunExpanded } from "./EndpointProbeRunExpanded";

type EndpointProbeView = ReturnType<typeof useEndpointProbeView>;

export function EndpointProbeResults({ view }: { view: EndpointProbeView }) {
  const batch = view.activeBatch;
  const promotableRun = view.promotableRun;
  const columns: Array<DataTableColumn<EndpointProbeRunSummary>> = [
    {
      key: "target",
      title: "站点 / 协议",
      render: (run) => (
        <div className="endpoint-probe-run-target">
          <strong>{run.name}</strong>
          <span>{endpointProbeInterfaceLabel(run.interface_type)}</span>
        </div>
      ),
    },
    {
      key: "model",
      title: "模型",
      render: (run) => <span className="endpoint-probe-model-name" title={run.model}>{run.model}</span>,
    },
    {
      key: "status",
      title: "状态",
      width: 76,
      align: "center",
      render: (run) => (
        <Badge tone={endpointProbeStatusTone(run.status)}>
          {endpointProbeStatusLabel(run.status)}
        </Badge>
      ),
    },
    {
      key: "result",
      title: "结果说明",
      width: 156,
      render: (run) => <RunResultNote run={run} />,
    },
    {
      key: "ttft",
      title: <MetricHelp helpKey="ttft">TTFT</MetricHelp>,
      width: 72,
      align: "right",
      render: (run) => formatMilliseconds(run.ttft_ms),
    },
    {
      key: "latency",
      title: <MetricHelp helpKey="latency">耗时</MetricHelp>,
      width: 76,
      align: "right",
      render: (run) => formatMilliseconds(run.latency_ms),
    },
    {
      key: "tokens",
      title: "Token",
      width: 68,
      align: "right",
      render: (run) => run.total_tokens.toLocaleString("zh-CN"),
    },
  ];

  return (
    <Card className="endpoint-probe-result-card">
      <div className="endpoint-probe-panel-head endpoint-probe-result-head">
        <div>
          <h2>测活结果</h2>
          <p>{batch ? batch.name : "启动测活后，这里会按请求展示真实响应。"}</p>
        </div>
        {batch && (
          <div className="endpoint-probe-result-head-actions">
            {promotableRun && (
              <Button
                icon={<Building2 size={15} />}
                variant="primary"
                onClick={() => void view.openPromotion(promotableRun.id)}
              >
                保存为服务商
              </Button>
            )}
            <Badge tone={endpointProbeStatusTone(batch.status)}>
              {endpointProbeStatusLabel(batch.status)}
            </Badge>
          </div>
        )}
      </div>

      {view.batchDetailError ? (
        <InlineAlert tone="danger" title="批次详情读取失败">
          {toErrorMessage(view.batchDetailError)}
        </InlineAlert>
      ) : view.batchDetailLoading && !batch ? (
        <LoadingBlock text="正在读取测活批次..." />
      ) : !batch ? (
        <EmptyState
          icon={<Network size={22} />}
          title="等待一次真实请求"
          description="可使用已保存服务商，或临时填写中转站；Streaming 开启后会实时显示服务端 SSE 输出。"
        />
      ) : (
        <>
          <div className="endpoint-probe-batch-overview">
            <MetricCard label="请求总数" value={batch.total_runs} />
            <MetricCard label="可用" value={batch.passed_runs} />
            <MetricCard label="失败" value={batch.failed_runs} />
            <MetricCard
              label="执行中"
              value={batch.pending_runs + batch.running_runs}
            />
          </div>
          <div className="endpoint-probe-batch-caption">
            <span>
              并发 {batch.concurrency} · {batch.streaming ? "Streaming" : "非流式"} ·
              {batch.save_body ? " 保存正文" : " 仅保存摘要"}
            </span>
            <span>{formatDate(batch.created_at)}</span>
          </div>
          <DataTable
            className="endpoint-probe-runs-table"
            columns={columns}
            expandable={{
              expandedRowKey: view.expandedRunId,
              expandOnRowClick: true,
              onExpandedRowChange: (key) => void view.expandRun(key ? String(key) : null),
              expandedRowRender: (run) => (
                <EndpointProbeRunExpanded
                  detail={view.runDetails[run.id]}
                  error={view.expandedRunId === run.id ? view.runDetailError : null}
                  liveText={view.streamText[run.id] ?? ""}
                  loading={view.loadingRunId === run.id}
                  run={run}
                  onCopy={view.copyProbeText}
                  onPromote={() => void view.openPromotion(run.id)}
                  onRetry={() => void view.expandRun(run.id)}
                />
              ),
            }}
            getRowKey={(run) => run.id}
            getRowClassName={(run) => `endpoint-probe-run-row is-${run.status}`}
            rows={batch.runs}
            scrollX={720}
          />
        </>
      )}
    </Card>
  );
}

function RunResultNote({ run }: { run: EndpointProbeRunSummary }) {
  const text = endpointProbeRunResultText(run);
  const tone = run.status === "failed"
    ? "failed"
    : run.status === "running" || run.status === "pending"
      ? "running"
      : canPromoteEndpointProbeRun(run)
        ? "promotable"
        : run.status === "passed"
          ? "passed"
          : "neutral";

  return (
    <span className={`endpoint-probe-run-result-note is-${tone}`} title={text}>
      {text}
    </span>
  );
}

function formatMilliseconds(value: number) {
  return value > 0 ? `${value.toLocaleString("zh-CN")}ms` : "-";
}

function formatDate(value: string) {
  return new Date(value).toLocaleString("zh-CN");
}

function toErrorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
