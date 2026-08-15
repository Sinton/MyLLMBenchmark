import { Card } from "../../../components/ui/Card";
import { ShieldCheck } from "../../../components/ui/icons";
import type { ModelSummary } from "../../../types/api";
import { ModelRow } from "./ProviderFeedback";

type CapabilityStats = {
  text_generation: number;
  embedding: number;
  multimodal: number;
  rerank: number;
};

type ProviderPanelsProps = {
  canScan: boolean;
  models: ModelSummary[];
  selectedModelCount: number;
  stats: CapabilityStats;
};

const capabilityItems = [
  { key: "text_generation", label: "文本生成", hint: "TTFT / TPS / Token 吞吐" },
  { key: "embedding", label: "向量嵌入", hint: "Batch / Text/s / Token/s" },
  { key: "multimodal", label: "视觉多模态", hint: "Image/s / 图文延迟" },
  { key: "rerank", label: "重排序", hint: "Query/s / Pair/s" },
] as const;

export function ProviderPanels({
  canScan,
  models,
  selectedModelCount,
  stats,
}: ProviderPanelsProps) {
  return (
    <div className="provider-panels">
      <Card
        title="模型能力概览"
        eyebrow="压测能力类型"
        action={<span className="rail-count">{selectedModelCount}</span>}
      >
        <p className="provider-section-subtitle">
          按压测指标口径分类，不按厂商、模型架构或接口协议分类。
        </p>
        <div className="capability-grid refined">
          {capabilityItems.map((item) => (
            <div key={item.key}>
              <span>{item.label}</span>
              <strong>{stats[item.key]}</strong>
              <em>{item.hint}</em>
            </div>
          ))}
        </div>

        <div className="provider-model-section">
          <div className="provider-section-heading">
            <h3>模型列表</h3>
            <span>{models.length ? `${models.length} 个可用模型` : "等待扫描"}</span>
          </div>
          {models.length ? (
            <div className="model-list">
              {models.map((model) => (
                <ModelRow key={model.id} model={model} />
              ))}
            </div>
          ) : (
            <div className="model-placeholder subtle">
              <ShieldCheck size={20} />
              <div>
                <strong>{canScan ? "等待模型扫描" : "先完成连接测试"}</strong>
                <span>
                  {canScan
                    ? "连接通过后会自动扫描，也可以点击上方的扫描模型按钮。"
                    : "请先测试连接，确认 Base URL、协议和 API Key 可用。"}
                </span>
              </div>
            </div>
          )}
        </div>
      </Card>

      <Card title="连接策略" eyebrow="安全与数据">
        <div className="policy-list readonly">
          <div>
            <span>密钥策略</span>
            <strong>本地明文展示</strong>
          </div>
          <div>
            <span>请求保存</span>
            <strong>默认不落盘</strong>
          </div>
          <div>
            <span>删除行为</span>
            <strong>清理关联模拟数据</strong>
          </div>
        </div>
      </Card>
    </div>
  );
}
