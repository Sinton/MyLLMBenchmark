import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import { normalizeModelType } from "../../../lib/modelTaxonomy";
import type { BenchmarkTaskSummary } from "../../../types/api";
import type { WorkbenchForm } from "../types";

export function useWorkbenchData(form: WorkbenchForm, activeTask?: BenchmarkTaskSummary | null) {
  const { data: providers = [] } = useQuery({
    queryKey: queryKeys.providers(),
    queryFn: api.listProviders,
  });

  const { data: datasets = [] } = useQuery({
    queryKey: queryKeys.datasets(),
    queryFn: api.listDatasets,
  });

  const { data: providerModels = [] } = useQuery({
    queryKey: queryKeys.providerModels(form.provider_id),
    queryFn: () => api.listProviderModels(form.provider_id),
    enabled: Boolean(form.provider_id),
  });

  const {
    data: providerDiagnostics = null,
    isFetching: providerDiagnosticsFetching,
  } = useQuery({
    queryKey: queryKeys.providerDiagnostics(form.provider_id),
    queryFn: () => api.getProviderDiagnostics(form.provider_id),
    enabled: Boolean(form.provider_id),
  });

  const selectedProvider = useMemo(
    () => providers.find((provider) => provider.id === form.provider_id),
    [form.provider_id, providers],
  );
  const selectedModel = useMemo(
    () => providerModels.find((model) => model.id === form.model_id),
    [form.model_id, providerModels],
  );
  const selectedDataset = useMemo(
    () => datasets.find((dataset) => dataset.id === form.dataset_id),
    [datasets, form.dataset_id],
  );

  const selectedModelType = normalizeModelType(
    selectedModel?.model_type ?? "text_generation",
  );
  const activeModelType = normalizeModelType(activeTask?.model_type ?? selectedModelType);

  return {
    providers,
    datasets,
    providerModels,
    selectedProvider,
    selectedModel,
    selectedDataset,
    providerDiagnostics,
    providerDiagnosticsFetching,
    selectedModelType,
    activeModelType,
  };
}
