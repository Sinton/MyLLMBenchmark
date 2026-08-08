import { Card } from "../../../components/ui/Card";
import { Play, Square } from "../../../components/ui/icons";
import { Button } from "../../../components/ui/Button";
import type { useEndpointProbeView } from "../hooks/useEndpointProbeView";
import { EndpointProbeBatchTargets } from "./EndpointProbeBatchTargets";
import { EndpointProbeCommonSettings } from "./EndpointProbeCommonSettings";

type EndpointProbeView = ReturnType<typeof useEndpointProbeView>;

export function EndpointProbeConfiguration({ view }: { view: EndpointProbeView }) {
  const isActive =
    view.activeBatch?.status === "pending" || view.activeBatch?.status === "running";
  const isStarting = view.running && !isActive;
  const requestCount = isActive
    ? view.activeBatch?.total_runs ?? 0
    : view.selectedRunCount;
  const requestSummary = `${requestCount} 个请求 · ${
    view.common.streaming ? "流式响应" : "非流式响应"
  } · ${view.common.save_body ? "保存正文" : "仅保存摘要"}`;
  const launchState = isActive || isStarting
    ? {
        description: requestSummary,
        title: view.stopping
          ? "正在停止测活"
          : isStarting
            ? "正在启动测活"
            : "测活进行中",
        tone: "running",
      }
    : view.startIssue
      ? {
          description: view.startIssue,
          title: view.listenersReady ? "配置未就绪" : "正在准备",
          tone: view.listenersReady ? "warning" : "pending",
        }
      : {
          description: requestSummary,
          title: "配置就绪",
          tone: "ready",
        };

  return (
    <Card className="endpoint-probe-config-card">
      <div className="endpoint-probe-panel-head">
        <div>
          <h2>测活配置</h2>
          <p>选择已保存服务商和模型，用同一 Prompt 批量确认协议、Key 与模型是否可用。</p>
        </div>
      </div>

      <div className="endpoint-probe-config-scroll">
        <EndpointProbeBatchTargets view={view} />
        <EndpointProbeCommonSettings view={view} />
      </div>

      <div className="endpoint-probe-launch-dock">
        <div
          aria-live="polite"
          className={`endpoint-probe-launch-summary is-${launchState.tone}`}
        >
          <div className="endpoint-probe-launch-status">
            <span aria-hidden="true" className="endpoint-probe-launch-status-dot" />
            <strong>{launchState.title}</strong>
          </div>
          <span title={launchState.description}>
            {launchState.description}
          </span>
        </div>
        {isActive ? (
          <Button
            loading={view.stopping}
            icon={<Square size={15} />}
            variant="danger"
            onClick={view.stop}
          >
            停止当前批次
          </Button>
        ) : (
          <Button
            disabled={Boolean(view.startIssue)}
            icon={<Play size={16} />}
            loading={view.running}
            variant="primary"
            onClick={view.start}
          >
            开始测活
          </Button>
        )}
      </div>
    </Card>
  );
}
