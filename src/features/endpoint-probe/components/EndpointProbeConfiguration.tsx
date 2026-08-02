import { Card } from "../../../components/ui/Card";
import { Play, Square } from "../../../components/ui/icons";
import { Button } from "../../../components/ui/Button";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { Tabs } from "../../../components/ui/Tabs";
import type { useEndpointProbeView } from "../hooks/useEndpointProbeView";
import { EndpointProbeBatchTargets } from "./EndpointProbeBatchTargets";
import { EndpointProbeCommonSettings } from "./EndpointProbeCommonSettings";
import { EndpointProbeSingleTarget } from "./EndpointProbeSingleTarget";

type EndpointProbeView = ReturnType<typeof useEndpointProbeView>;

export function EndpointProbeConfiguration({ view }: { view: EndpointProbeView }) {
  const isActive = view.activeBatch?.status === "pending" || view.activeBatch?.status === "running";

  return (
    <Card className="endpoint-probe-config-card">
      <div className="endpoint-probe-panel-head">
        <div>
          <h2>测活配置</h2>
          <p>使用真实端点发出最小请求，快速确认协议、Key 与模型是否可用。</p>
        </div>
        <Tabs
          ariaLabel="测活模式"
          items={[
            { key: "single", label: "单次测活" },
            { key: "batch", label: "批量测活" },
          ]}
          value={view.workspaceMode}
          onChange={view.setWorkspaceMode}
        />
      </div>

      <div className="endpoint-probe-config-scroll">
        {view.workspaceMode === "single" ? (
          <EndpointProbeSingleTarget view={view} />
        ) : (
          <EndpointProbeBatchTargets view={view} />
        )}
        <EndpointProbeCommonSettings view={view} />
      </div>

      <div className="endpoint-probe-launch-dock">
        <div className="endpoint-probe-launch-summary">
          <strong>
            {view.workspaceMode === "batch"
              ? `${view.selectedRunCount} 个模型请求`
              : "1 个模型请求"}
          </strong>
          <span>
            {view.common.streaming ? "实时 Streaming" : "非流式响应"}
            {view.common.save_body ? " · 保存正文" : " · 仅保存摘要"}
          </span>
        </div>
        {view.startIssue && !isActive && (
          <InlineAlert tone="info">{view.startIssue}</InlineAlert>
        )}
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
            {view.workspaceMode === "batch" ? "开始批量测活" : "开始测活"}
          </Button>
        )}
      </div>
    </Card>
  );
}
