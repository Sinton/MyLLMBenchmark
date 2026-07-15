import type { Dispatch, SetStateAction } from "react";
import { Input } from "../../../components/ui/Input";
import { Toggle } from "../../../components/ui/Toggle";
import { SelectField } from "../../../components/ui/SelectField";
import { getModelTypeLabel } from "../../../lib/modelTaxonomy";
import type { WorkbenchForm } from "../types";

type WorkloadConfigSectionProps = {
  form: WorkbenchForm;
  modelType: string;
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
};

export function WorkloadConfigSection({
  form,
  modelType,
  setForm,
}: WorkloadConfigSectionProps) {
  const title = getModelTypeLabel(modelType);

  return (
    <div className="workload-config-panel">
      <div className="workload-config-title">
        <span>专项负载</span>
        <strong>{title}</strong>
      </div>

      {modelType === "embedding" ? (
        <div className="form-grid">
          <Input
            label="Batch Size"
            min={1}
            type="number"
            value={form.embedding_batch_size}
            onChange={(event) =>
              setForm((current) => ({
                ...current,
                embedding_batch_size: Number(event.target.value),
                embedding_text_count_per_request: Number(event.target.value),
              }))
            }
          />
          <Input
            label="文本/请求"
            min={1}
            type="number"
            value={form.embedding_text_count_per_request}
            onChange={(event) =>
              setForm((current) => ({
                ...current,
                embedding_text_count_per_request: Number(event.target.value),
              }))
            }
          />
        </div>
      ) : modelType === "rerank" ? (
        <div className="form-grid">
          <Input
            label="Docs/Query"
            min={1}
            type="number"
            value={form.rerank_documents_per_query}
            onChange={(event) =>
              setForm((current) => ({
                ...current,
                rerank_documents_per_query: Number(event.target.value),
              }))
            }
          />
          <Input
            label="TopK"
            min={1}
            type="number"
            value={form.rerank_top_k}
            onChange={(event) =>
              setForm((current) => ({
                ...current,
                rerank_top_k: Number(event.target.value),
              }))
            }
          />
        </div>
      ) : modelType === "multimodal" ? (
        <>
          <SelectField
            label="图片尺寸档位"
            onChange={(vision_image_profile) =>
              setForm((current) => ({ ...current, vision_image_profile }))
            }
            options={[
              { value: "small", label: "Small", description: "低分辨率图片" },
              { value: "medium", label: "Medium", description: "常规业务图片" },
              { value: "large", label: "Large", description: "高分辨率图片" },
            ]}
            value={form.vision_image_profile}
          />
          <Input
            label="图片数/请求"
            min={1}
            type="number"
            value={form.vision_image_count}
            onChange={(event) =>
              setForm((current) => ({
                ...current,
                vision_image_count: Number(event.target.value),
              }))
            }
          />
        </>
      ) : (
        <>
          <div className="form-grid">
            <Input
              label="Max Output"
              min={1}
              type="number"
              value={form.max_output_tokens}
              onChange={(event) =>
                setForm((current) => ({
                  ...current,
                  max_output_tokens: Number(event.target.value),
                }))
              }
            />
            <div className="workload-toggle-field">
              <span className="workload-toggle-label">Streaming</span>
              <div className="workload-toggle-shell">
                <span>{form.streaming ? "已开启" : "已关闭"}</span>
                <Toggle
                  ariaLabel="Streaming"
                  checked={form.streaming}
                  onChange={(streaming) =>
                    setForm((current) => ({ ...current, streaming }))
                  }
                />
              </div>
            </div>
          </div>
          <SelectField
            label="Prompt 档位"
            onChange={(prompt_profile) =>
              setForm((current) => ({ ...current, prompt_profile }))
            }
            options={[
              { value: "short", label: "Short", description: "短 Prompt" },
              { value: "mixed", label: "Mixed", description: "混合长度" },
              { value: "long", label: "Long", description: "长上下文" },
            ]}
            value={form.prompt_profile}
          />
        </>
      )}
    </div>
  );
}
