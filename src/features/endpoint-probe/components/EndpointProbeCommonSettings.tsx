import { useEffect } from "react";
import { Button } from "../../../components/ui/Button";
import { Input } from "../../../components/ui/Input";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { Popover } from "../../../components/ui/Popover";
import { SelectField } from "../../../components/ui/SelectField";
import { Textarea } from "../../../components/ui/Textarea";
import { Toggle } from "../../../components/ui/Toggle";
import { Plus, Save, Wrench } from "../../../components/ui/icons";
import type { useEndpointProbeView } from "../hooks/useEndpointProbeView";

type EndpointProbeView = ReturnType<typeof useEndpointProbeView>;

export function EndpointProbeCommonSettings({ view }: { view: EndpointProbeView }) {
  const selectedProvider = view.providers.find((provider) => provider.id === view.singleProviderId);
  const interfaceType = view.singleSource === "temporary"
    ? view.temporary.interface_type
    : selectedProvider?.interface_type;
  const temperatureMax = interfaceType === "Anthropic" ? 1 : 2;
  const currentTemperature = view.common.temperature;
  const setCommon = view.setCommon;

  const update = <K extends keyof typeof view.common>(
    key: K,
    value: (typeof view.common)[K],
  ) => view.setCommon((current) => ({ ...current, [key]: value }));

  useEffect(() => {
    if (currentTemperature <= temperatureMax) return;
    setCommon((current) => ({ ...current, temperature: temperatureMax }));
  }, [currentTemperature, temperatureMax, setCommon]);

  const settingSummary = [
    view.common.streaming ? "Stream 开启" : "Stream 关闭",
    `Temp ${formatTemperature(view.common.temperature)}`,
    `${view.common.max_output_tokens} Token`,
    `${view.common.timeout_seconds}s`,
    view.common.save_body ? "保存正文" : "仅摘要",
  ].join(" · ");

  return (
    <section className="endpoint-probe-config-section endpoint-probe-common-settings">
      <div className="endpoint-probe-section-title endpoint-probe-section-title-with-action">
        <div className="endpoint-probe-section-heading-copy">
          <span>请求设置</span>
          <small>{settingSummary}</small>
        </div>
        <Popover
          align="end"
          className="endpoint-probe-advanced-popover"
          trigger={({ contentId, open, toggle }) => (
            <Button
              aria-controls={contentId}
              aria-expanded={open}
              aria-haspopup="dialog"
              aria-label={open ? "收起请求参数" : "展开请求参数"}
              className={`endpoint-probe-advanced-trigger${open ? " is-open" : ""}`}
              icon={<Wrench size={15} />}
              onClick={toggle}
              title={open ? "收起请求参数" : "请求参数"}
              type="button"
              variant="ghost"
            />
          )}
        >
          <div
            aria-label="请求参数"
            className="endpoint-probe-advanced-panel"
            role="dialog"
          >
            <div className="endpoint-probe-advanced-fields">
              <Input
                label="Temperature"
                hint={
                  interfaceType === "Anthropic"
                    ? "Anthropic 范围 0-1，测活默认 0.2"
                    : "OpenAI / Responses 范围 0-2，测活默认 0.2"
                }
                max={temperatureMax}
                min={0}
                step={0.1}
                type="number"
                value={view.common.temperature}
                onChange={(event) =>
                  update(
                    "temperature",
                    clampTemperature(Number(event.target.value), temperatureMax),
                  )
                }
              />
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
            </div>
            <div className="endpoint-probe-toggle-list endpoint-probe-advanced-toggle-list">
              <div className="endpoint-probe-toggle-item">
                <div>
                  <strong>Stream</strong>
                  <span>通过 SSE 增量接收响应，不模拟逐字动画。</span>
                </div>
                <Toggle
                  ariaLabel="Stream"
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
          </div>
        </Popover>
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
    </section>
  );
}

function clampTemperature(value: number, max: number) {
  if (!Number.isFinite(value)) return 0;
  return Math.min(max, Math.max(0, Number(value.toFixed(2))));
}

function formatTemperature(value: number) {
  return Number.isInteger(value) ? value.toFixed(0) : value.toFixed(1);
}
