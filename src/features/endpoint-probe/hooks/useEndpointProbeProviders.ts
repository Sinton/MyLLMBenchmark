import {
  useEffect,
  useMemo,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import { useQueries, useQuery } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import type { EndpointProbeModelOption } from "../../../types/api";

export function useEndpointProbeProviders(
  batchExtraModels: Record<string, string[]>,
) {
  const [singleProviderId, setSingleProviderIdState] = useState("");
  const [singleProviderModel, setSingleProviderModel] = useState("");
  const [requestedProviderIds, setRequestedProviderIds] = useState<string[]>([]);
  const providersQuery = useQuery({
    queryKey: queryKeys.providers(),
    queryFn: api.listProviders,
  });
  const providers = useMemo(
    () =>
      (providersQuery.data ?? []).filter((provider) =>
        ["OpenAI", "OpenAI-Response", "Anthropic"].includes(provider.interface_type),
      ),
    [providersQuery.data],
  );
  const providerModelQueries = useQueries({
    queries: providers.map((provider) => ({
      queryKey: queryKeys.providerModels(provider.id),
      queryFn: () => api.listProviderModels(provider.id),
      enabled: requestedProviderIds.includes(provider.id),
    })),
  });
  const providerModels = useMemo(() => {
    const map: Record<string, EndpointProbeModelOption[]> = {};
    providers.forEach((provider, index) => {
      const persisted = providerModelQueries[index]?.data ?? [];
      const extras = (batchExtraModels[provider.id] ?? []).map<EndpointProbeModelOption>(
        (name) => ({
          name,
          model_type: "text_generation",
          capabilities: ["chat"],
          supports_streaming: true,
        }),
      );
      map[provider.id] = dedupeModels([...persisted, ...extras]);
    });
    return map;
  }, [batchExtraModels, providerModelQueries, providers]);

  useEffect(() => {
    if (!singleProviderId && providers[0]) setSingleProviderIdState(providers[0].id);
  }, [providers, singleProviderId]);

  useEffect(() => {
    if (!singleProviderId) return;
    requestProviderModels(setRequestedProviderIds, singleProviderId);
  }, [singleProviderId]);

  const singleProviderModels = providerModels[singleProviderId] ?? [];
  useEffect(() => {
    if (singleProviderModels.some((model) => model.name === singleProviderModel)) return;
    setSingleProviderModel(singleProviderModels[0]?.name ?? "");
  }, [singleProviderId, singleProviderModel, singleProviderModels]);

  return {
    ensureProviderModels: (providerId: string) =>
      requestProviderModels(setRequestedProviderIds, providerId),
    providerModels,
    providers,
    providersLoading: providersQuery.isFetching,
    singleProviderId,
    singleProviderModel,
    singleProviderModels,
    setSingleProviderId: (providerId: string) => {
      setSingleProviderIdState(providerId);
      setSingleProviderModel("");
    },
    setSingleProviderModel,
  };
}

function requestProviderModels(
  setRequestedProviderIds: Dispatch<SetStateAction<string[]>>,
  providerId: string,
) {
  setRequestedProviderIds((current) =>
    current.includes(providerId) ? current : [...current, providerId],
  );
}

function dedupeModels(models: EndpointProbeModelOption[]) {
  const seen = new Set<string>();
  return models.filter((model) => {
    const key = model.name.toLowerCase();
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}
