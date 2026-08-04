import { type FormEvent, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import { useNotification } from "../../../components/ui/Notification";
import {
  countCapabilities,
  DEFAULT_INTERFACE_TYPE,
  getErrorMessage,
} from "../domain/providerView";
import type {
  ProviderDiagnosticsResult,
  ProviderInterfaceType,
  ProviderSummary,
} from "../../../types/api";
import { PROVIDER_INTERFACE_TYPES } from "../../../types/api";

type ProviderFormState = {
  name: string;
  base_url: string;
  api_key: string;
  interface_type: ProviderInterfaceType;
};

type ProviderDrawerMode = "create" | "edit" | null;

const emptyProviderForm = (): ProviderFormState => ({
  name: "",
  base_url: "",
  api_key: "",
  interface_type: DEFAULT_INTERFACE_TYPE,
});

export function useProvidersController() {
  const queryClient = useQueryClient();
  const { notify } = useNotification();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [drawerMode, setDrawerMode] = useState<ProviderDrawerMode>(null);
  const [editingProviderId, setEditingProviderId] = useState<string | null>(null);
  const [recentConnectedProviderId, setRecentConnectedProviderId] = useState<string | null>(null);
  const [connectionFailure, setConnectionFailure] = useState<{
    providerId: string;
    message: string;
  } | null>(null);
  const [diagnosticsResult, setDiagnosticsResult] =
    useState<ProviderDiagnosticsResult | null>(null);
  const [form, setForm] = useState<ProviderFormState>(emptyProviderForm);

  const { data: providers = [] } = useQuery({
    queryKey: queryKeys.providers(),
    queryFn: api.listProviders,
  });

  const selected = useMemo(
    () => providers.find((provider) => provider.id === selectedId) ?? providers[0],
    [providers, selectedId],
  );
  const selectedProviderId = selected?.id ?? "";

  const { data: models = [], isFetching: isModelsFetching } = useQuery({
    queryKey: queryKeys.providerModels(selectedProviderId),
    queryFn: () => api.listProviderModels(selectedProviderId),
    enabled: Boolean(selectedProviderId),
  });

  const { data: savedDiagnostics = null } = useQuery({
    queryKey: queryKeys.providerDiagnostics(selectedProviderId),
    queryFn: () => api.getProviderDiagnostics(selectedProviderId),
    enabled: Boolean(selectedProviderId),
  });

  const capabilityStats = useMemo(() => countCapabilities(models), [models]);
  const selectedModelCount = models.length || selected?.model_count || 0;

  const invalidateProviderViews = async () => {
    await queryClient.invalidateQueries({ queryKey: queryKeys.providers() });
    await queryClient.invalidateQueries({ queryKey: queryKeys.dashboard() });
  };

  const openCreateProvider = () => {
    setForm(emptyProviderForm());
    setEditingProviderId(null);
    setDrawerMode("create");
  };

  const closeProviderDrawer = () => {
    setDrawerMode(null);
    setEditingProviderId(null);
    setForm(emptyProviderForm());
  };

  const openEditProvider = (provider: ProviderSummary) => {
    setForm({
      name: provider.name,
      base_url: provider.base_url_masked,
      api_key: provider.api_key_masked === "未配置" ? "" : provider.api_key_masked,
      interface_type: normalizeProviderInterfaceType(provider.interface_type),
    });
    setEditingProviderId(provider.id);
    setDrawerMode("edit");
  };

  const setIsCreating = (open: boolean) => {
    if (open) {
      openCreateProvider();
    } else {
      closeProviderDrawer();
    }
  };

  const createMutation = useMutation({
    mutationFn: api.createProvider,
    onSuccess: async (provider) => {
      setSelectedId(provider.id);
      closeProviderDrawer();
      setRecentConnectedProviderId(null);
      setConnectionFailure(null);
      setDiagnosticsResult(null);
      await invalidateProviderViews();
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ providerId, input }: { providerId: string; input: ProviderFormState }) =>
      api.updateProvider(providerId, input),
    onSuccess: async (provider) => {
      setSelectedId(provider.id);
      closeProviderDrawer();
      setRecentConnectedProviderId(null);
      setConnectionFailure(null);
      setDiagnosticsResult(null);
      await invalidateProviderViews();
      await queryClient.invalidateQueries({
        queryKey: queryKeys.providerModels(provider.id),
      });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: api.deleteProvider,
    onSuccess: async () => {
      setSelectedId(null);
      setRecentConnectedProviderId(null);
      setConnectionFailure(null);
      await invalidateProviderViews();
      await queryClient.invalidateQueries({ queryKey: queryKeys.reports() });
    },
  });

  const scanModelsMutation = useMutation({
    mutationFn: api.scanProviderModels,
    onSuccess: async (result) => {
      notify({
        title: "模型列表已更新",
        description: `已扫描到 ${result.models.length} 个模型`,
        tone: "success",
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.providerModels(result.provider_id),
      });
      await invalidateProviderViews();
    },
  });

  const testConnectionMutation = useMutation({
    mutationFn: api.testProviderConnection,
    onMutate: () => {
      setConnectionFailure(null);
      setRecentConnectedProviderId(null);
    },
    onSuccess: async (result) => {
      await invalidateProviderViews();
      if (result.ok) {
        setConnectionFailure(null);
        setRecentConnectedProviderId(result.provider_id);
        notify({
          title: "连接测试通过",
          description: `${result.message} 正在继续扫描模型。`,
          tone: "success",
        });
        scanModelsMutation.mutate(result.provider_id);
        return;
      }
      setRecentConnectedProviderId(null);
      setConnectionFailure({
        providerId: result.provider_id,
        message: result.message,
      });
    },
  });

  const diagnosticsMutation = useMutation({
    mutationFn: api.diagnoseProvider,
    onMutate: () => setDiagnosticsResult(null),
    onSuccess: async (result) => {
      setDiagnosticsResult(result);
      await queryClient.invalidateQueries({
        queryKey: queryKeys.providerDiagnostics(result.provider_id),
      });
    },
  });

  const isScanningCurrent = Boolean(
    selected &&
      scanModelsMutation.isPending &&
      scanModelsMutation.variables === selected.id,
  );
  const canScanSelected = Boolean(
    selected &&
      (selected.status === "online" || recentConnectedProviderId === selected.id),
  );
  const showCreatePanel = Boolean(drawerMode);
  const showEmptyOnboarding = providers.length === 0 && !drawerMode;
  const providerDrawerMode = drawerMode ?? "create";

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (!form.name.trim() || !form.base_url.trim()) return;
    if (drawerMode === "edit" && editingProviderId) {
      updateMutation.mutate({ providerId: editingProviderId, input: form });
      return;
    }
    createMutation.mutate(form);
  };

  return {
    canScanSelected,
    capabilityStats,
    createPending: createMutation.isPending,
    deleteError: deleteMutation.isError ? deleteMutation.error : undefined,
    deleting: deleteMutation.isPending,
    diagnosticsError:
      diagnosticsMutation.isError &&
      selected &&
      diagnosticsMutation.variables?.provider_id === selected.id
        ? diagnosticsMutation.error
        : undefined,
    diagnosticsPending: diagnosticsMutation.isPending,
    diagnosticsResult: diagnosticsResult ?? savedDiagnostics,
    form,
    getErrorMessage,
    isModelsFetching,
    isCreating: drawerMode === "create",
    isEditing: drawerMode === "edit",
    isScanningCurrent,
    models,
    providers,
    scanError:
      scanModelsMutation.isError &&
      selected &&
      scanModelsMutation.variables === selected.id
        ? scanModelsMutation.error
        : undefined,
    selected,
    selectedModelCount,
    closeProviderDrawer,
    editPending: updateMutation.isPending,
    openEditProvider,
    providerDrawerMode,
    setForm,
    setIsCreating,
    setSelectedId,
    showCreatePanel,
    showEmptyOnboarding,
    submit,
    testError:
      testConnectionMutation.isError &&
      selected &&
      testConnectionMutation.variables === selected.id
        ? testConnectionMutation.error
        : selected &&
            connectionFailure &&
            connectionFailure.providerId === selected.id
          ? connectionFailure.message
          : undefined,
    testPending: testConnectionMutation.isPending,
    onDelete: async () => {
      if (!selected) return;
      await deleteMutation.mutateAsync(selected.id);
    },
    onScan: () => selected && scanModelsMutation.mutate(selected.id),
    onTestConnection: () =>
      selected && testConnectionMutation.mutate(selected.id),
    onDiagnose: () =>
      selected &&
      diagnosticsMutation.mutate({
        provider_id: selected.id,
      }),
  };
}

function normalizeProviderInterfaceType(value: string): ProviderInterfaceType {
  return PROVIDER_INTERFACE_TYPES.includes(value as ProviderInterfaceType)
    ? (value as ProviderInterfaceType)
    : DEFAULT_INTERFACE_TYPE;
}
