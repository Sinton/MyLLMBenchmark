import type { Dispatch, SetStateAction } from "react";
import { SelectField } from "../../../components/common/SelectField";
import {
  getModelCapabilities,
  getModelTypeLabel,
  MODEL_CAPABILITY_LABELS,
} from "../../../lib/modelTaxonomy";
import type { DatasetSummary, ModelSummary, ProviderSummary } from "../../../types/api";
import type { WorkbenchForm } from "../types";

type TargetSelectorSectionProps = {
  datasets: DatasetSummary[];
  form: WorkbenchForm;
  providerModels: ModelSummary[];
  providers: ProviderSummary[];
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
};

export function TargetSelectorSection({
  datasets,
  form,
  providerModels,
  providers,
  setForm,
}: TargetSelectorSectionProps) {
  return (
    <>
      <SelectField
        label="模型服务商"
        onChange={(provider_id) => setForm({ ...form, provider_id, model_id: "" })}
        options={providers.map((provider) => ({
          value: provider.id,
          label: provider.name,
          description: provider.interface_type,
        }))}
        value={form.provider_id}
      />
      <SelectField
        label="模型"
        disabled={!providerModels.length}
        onChange={(model_id) => setForm({ ...form, model_id })}
        options={
          providerModels.length
            ? providerModels.map((model) => ({
                value: model.id,
                label: model.name,
                description: [
                  getModelTypeLabel(model.model_type),
                  ...getModelCapabilities(model).map(
                    (capability) => MODEL_CAPABILITY_LABELS[capability],
                  ),
                ].join(" · "),
              }))
            : [{ value: "", label: "先在服务商页扫描模型" }]
        }
        value={form.model_id}
      />
      <SelectField
        label="数据集"
        onChange={(dataset_id) => setForm({ ...form, dataset_id })}
        options={datasets.map((dataset) => ({
          value: dataset.id,
          label: dataset.name,
          description: getModelTypeLabel(dataset.dataset_type),
        }))}
        value={form.dataset_id}
      />
    </>
  );
}
