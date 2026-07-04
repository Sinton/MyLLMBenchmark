import { useEffect, type Dispatch, type SetStateAction } from "react";
import type { DatasetSummary, ModelSummary, ProviderSummary } from "../../../types/api";
import {
  datasetTypeForModel,
  normalizeDatasetType,
} from "../domain/datasetMatching";
import type { WorkbenchForm } from "../types";

type UseWorkbenchFormSyncInput = {
  datasets: DatasetSummary[];
  form: WorkbenchForm;
  providerModels: ModelSummary[];
  providers: ProviderSummary[];
  selectedModelType: string;
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
};

export function useWorkbenchFormSync({
  datasets,
  form,
  providerModels,
  providers,
  selectedModelType,
  setForm,
}: UseWorkbenchFormSyncInput) {
  useEffect(() => {
    if (!form.provider_id && providers[0]) {
      setForm((current) => ({ ...current, provider_id: providers[0].id }));
    }
    if (!form.dataset_id && datasets[0]) {
      setForm((current) => ({ ...current, dataset_id: datasets[0].id }));
    }
  }, [datasets, form.dataset_id, form.provider_id, providers, setForm]);

  useEffect(() => {
    if (providerModels[0] && !form.model_id) {
      setForm((current) => ({ ...current, model_id: providerModels[0].id }));
    }
  }, [form.model_id, providerModels, setForm]);

  useEffect(() => {
    const datasetType = datasetTypeForModel(selectedModelType);
    const matched = datasets.find(
      (dataset) => normalizeDatasetType(dataset.dataset_type) === datasetType,
    );
    if (matched && form.dataset_id !== matched.id) {
      setForm((current) => ({ ...current, dataset_id: matched.id }));
    }
  }, [datasets, form.dataset_id, selectedModelType, setForm]);
}
