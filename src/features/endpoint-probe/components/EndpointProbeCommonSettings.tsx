import { Button } from "../../../components/ui/Button";
import { Input } from "../../../components/ui/Input";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { SelectField } from "../../../components/ui/SelectField";
import { Textarea } from "../../../components/ui/Textarea";
import { Toggle } from "../../../components/ui/Toggle";
import { Plus, Save } from "../../../components/ui/icons";
import type { useEndpointProbeView } from "../hooks/useEndpointProbeView";

type EndpointProbeView = ReturnType<typeof useEndpointProbeView>;

export function EndpointProbeCommonSettings({ view }: { view: EndpointProbeView }) {
  const update = <K extends keyof typeof view.common>(
    key: K,
    value: (typeof view.common)[K],
  ) => view.setCommon((current) => ({ ...current, [key]: value }));

  return (
    <section className="endpoint-probe-config-section endpoint-probe-common-settings">
      <div className="endpoint-probe-section-title">
        <span>请求设置</span>
        <small>当前批次的所有模型共用</small>
      </div>
      <div className="endpoint-probe-prompt-template-row">
        <SelectField
          ariaLabel="Prompt 模板"
          disabled={view.promptTemplatesLoading || view.running}
          value={view.selectedPromptTemplateId}
          options={view.promptTemplates.map((template) => ({
            value: template.id,
            label: template.name,
            description: template.prompt,
          }))}
          onChange={view.selectPromptTemplate}
        />
        <Button
          aria-label="保存当前 Prompt 模板"
          className="endpoint-probe-icon-action"
          disabled={!view.promptTemplateDirty || view.running}
          icon={<Save size={15} />}
          loading={view.savingPromptTemplate}
          onClick={view.saveCurrentPromptTemplate}
          title="保存模板"
          type="button"
        />
        <Button
          aria-label="新增 Prompt 模板"
          className="endpoint-probe-icon-action"
          disabled={view.running}
          icon={<Plus size={15} />}
          onClick={view.addPromptTemplate}
          title="新增模板"
          type="button"
        />
      </div>
      <Textarea
        label="测试 Prompt"
        rows={4}
        value={view.common.prompt}
        onChange={(event) => update("prompt", event.target.value)}
      />
      <div className={`endpoint-probe-form-grid ${view.workspaceMode === "batch" ? "three" : "two"}`}>
        <Input
          label="最大输出 Token"
          hint="限制模型本次最多生成的 Token 数"
          max={8192}
          min={1}
          type="number"
          value={view.common.max_output_tokens}
          onChange={(event) => update("max_output_tokens", Number(event.target.value))}
        />
        <Input
          label="请求超时（秒）"
          max={600}
          min={5}
          type="number"
          value={view.common.timeout_seconds}
          onChange={(event) => update("timeout_seconds", Number(event.target.value))}
        />
        {view.workspaceMode === "batch" && (
          <Input
            label="并发请求"
            hint="允许 1-10 个请求同时执行"
            max={10}
            min={1}
            type="number"
            value={view.common.concurrency}
            onChange={(event) => update("concurrency", Number(event.target.value))}
          />
        )}
      </div>
      <div className="endpoint-probe-toggle-list">
        <div className="endpoint-probe-toggle-item">
          <div>
            <strong>Streaming 实时响应</strong>
            <span>展示服务端真实 SSE chunk，不模拟逐字动画。</span>
          </div>
          <Toggle
            ariaLabel="Streaming 实时响应"
            checked={view.common.streaming}
            onChange={(checked) => update("streaming", checked)}
          />
        </div>
        <div className="endpoint-probe-toggle-item">
          <div>
            <strong>保存 Prompt / 响应正文</strong>
            <span>关闭时历史只保留摘要；当前会话仍可查看完整返回。</span>
          </div>
          <Toggle
            ariaLabel="保存 Prompt 和响应正文"
            checked={view.common.save_body}
            onChange={(checked) => update("save_body", checked)}
          />
        </div>
      </div>
      {view.common.save_body && (
        <InlineAlert tone="warning" title="正文可能包含敏感数据">
          本批次的 Prompt 与响应将写入本地数据目录，请确认内容允许留存。
        </InlineAlert>
      )}
    </section>
  );
}
