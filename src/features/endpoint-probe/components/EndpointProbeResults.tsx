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
  const runCount = batch?.runs.length ?? 0;
  const showBatchOverview = runCount > 1;
  const singleRun = batch && runCount === 1 ? batch.runs[0] : null;
  const columns: Array<DataTableColumn<EndpointProbeRunSummary>> = [
    {
      key: "target",
      title: "站点 / 模型",
      render: (run) => (
        <div className="endpoint-probe-run-target">
          <strong>{run.name}</strong>
          <span title={`${endpointProbeInterfaceLabel(run.interface_type)} · ${run.model}`}>
            {endpointProbeInterfaceLabel(run.interface_type)} · {run.model}
          </span>
        </div>
      ),
    },
    {
      key: "result",
      title: "结果",
      width: 248,
      render: (run) => (
        <div
          className={[
            "endpoint-probe-run-outcome",
            view.expandedRunId === run.id ? "is-expanded" : "",
          ].filter(Boolean).join(" ")}
        >
          <Badge tone={endpointProbeStatusTone(run.status)}>
            {endpointProbeStatusLabel(run.status)}
          </Badge>
          {view.expandedRunId !== run.id && <RunResultNote run={run} />}
        </div>
      ),
    },
    {
      key: "metrics",
      title: <MetricHelp helpKey="latency">核心指标</MetricHelp>,
      width: 218,
      align: "right",
      render: (run) => (
        <div className="endpoint-probe-run-metric-strip">
          <span>
            <small>TTFT</small>
            <strong>{formatMilliseconds(run.ttft_ms)}</strong>
          </span>
          <span>
            <small>耗时</small>
            <strong>{formatMilliseconds(run.latency_ms)}</strong>
          </span>
          <span>
            <small>Token</small>
            <strong>{run.total_tokens.toLocaleString("zh-CN")}</strong>
          </span>
        </div>
      ),
    },
  ];

  return (
    <Card
      className={[
        "endpoint-probe-result-card",
        singleRun ? "is-single-run" : "",
      ].filter(Boolean).join(" ")}
    >
      <div className="endpoint-probe-panel-head endpoint-probe-result-head">
        <div>
          <h2>测活结果</h2>
          {batch && <p>{batch.name}</p>}
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
          description="可使用已保存服务商，或临时填写中转站；Stream 开启后会实时显示服务端 SSE 输出。"
        />
      ) : (
        <>
          {showBatchOverview && (
            <div className="endpoint-probe-batch-overview">
              <MetricCard label="请求总数" value={batch.total_runs} />
              <MetricCard label="可用" value={batch.passed_runs} />
              <MetricCard label="失败" value={batch.failed_runs} />
              <MetricCard
                label="执行中"
                value={batch.pending_runs + batch.running_runs}
              />
            </div>
          )}
          <div
            className={[
              "endpoint-probe-batch-caption",
              showBatchOverview ? "" : "is-compact",
            ].filter(Boolean).join(" ")}
          >
            <span>
              {showBatchOverview ? `并发 ${batch.concurrency} · ` : ""}
              {batch.streaming ? "Stream" : "非 Stream"} ·
              Temp {formatTemperature(batch.temperature)} ·
              {batch.save_body ? " 保存正文" : " 仅保存摘要"}
            </span>
            <span>{formatDate(batch.created_at)}</span>
          </div>
          {singleRun ? (
            <EndpointProbeSingleRunResult run={singleRun} view={view} />
          ) : (
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
                    onRetry={() => void view.expandRun(run.id)}
                  />
                ),
              }}
              getRowKey={(run) => run.id}
              getRowClassName={(run) => `endpoint-probe-run-row is-${run.status}`}
              rows={batch.runs}
            />
          )}
        </>
      )}
    </Card>
  );
}

function EndpointProbeSingleRunResult({
  run,
  view,
}: {
  run: EndpointProbeRunSummary;
  view: EndpointProbeView;
}) {
  return (
    <section className={`endpoint-probe-single-run-panel is-${run.status}`}>
      <div className="endpoint-probe-single-run-summary">
        <div className="endpoint-probe-run-target">
          <strong>{run.name}</strong>
          <span title={`${endpointProbeInterfaceLabel(run.interface_type)} · ${run.model}`}>
            {endpointProbeInterfaceLabel(run.interface_type)} · {run.model}
          </span>
        </div>
        <div className="endpoint-probe-single-run-status">
          <Badge tone={endpointProbeStatusTone(run.status)}>
            {endpointProbeStatusLabel(run.status)}
          </Badge>
        </div>
        <div className="endpoint-probe-run-metric-strip">
          <span>
            <small>TTFT</small>
            <strong>{formatMilliseconds(run.ttft_ms)}</strong>
          </span>
          <span>
            <small>耗时</small>
            <strong>{formatMilliseconds(run.latency_ms)}</strong>
          </span>
          <span>
            <small>Token</small>
            <strong>{run.total_tokens.toLocaleString("zh-CN")}</strong>
          </span>
        </div>
      </div>
      <div className="endpoint-probe-single-run-detail">
        <EndpointProbeRunExpanded
          detail={view.runDetails[run.id]}
          error={view.expandedRunId === run.id ? view.runDetailError : null}
          liveText={view.streamText[run.id] ?? ""}
          loading={view.loadingRunId === run.id}
          run={run}
          onCopy={view.copyProbeText}
          onRetry={() => void view.expandRun(run.id)}
        />
      </div>
    </section>
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

function formatTemperature(value: number) {
  return Number.isInteger(value) ? value.toFixed(0) : value.toFixed(1);
}

function toErrorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
