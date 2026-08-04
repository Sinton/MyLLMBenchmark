import { useEffect, useMemo, useRef, useState } from "react";
import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import { useNotification } from "../../../components/ui/Notification";
import { useToast } from "../../../components/ui/Toast";
import type {
  EndpointProbeBatchDetail,
  EndpointProbeModelOption,
  EndpointProbeModelScanInput,
  EndpointProbeRunDetail,
  EndpointProbeStartInput,
  ProviderImportItem,
  ProviderImportResult,
} from "../../../types/api";
import {
  buildEndpointProbeStartInput,
  countSelectedProbeRuns,
  createEndpointProbeCommonForm,
  createEndpointProbeTemporaryForm,
  validateEndpointProbeStart,
  type EndpointProbeSingleSource,
  type EndpointProbeWorkspaceMode,
} from "../domain/endpointProbeForm";
import { useEndpointProbeEvents } from "./useEndpointProbeEvents";
import { useEndpointProbeProviders } from "./useEndpointProbeProviders";

export function useEndpointProbeView() {
  const queryClient = useQueryClient();
  const { notify } = useNotification();
  const { showToast } = useToast();
  const [workspaceMode, setWorkspaceMode] =
    useState<EndpointProbeWorkspaceMode>("single");
  const [singleSource, setSingleSource] =
    useState<EndpointProbeSingleSource>("provider");
  const [common, setCommon] = useState(createEndpointProbeCommonForm);
  const [temporary, setTemporary] = useState(createEndpointProbeTemporaryForm);
  const [temporaryModels, setTemporaryModels] = useState<EndpointProbeModelOption[]>([]);
  const [batchModels, setBatchModels] = useState<Record<string, string[]>>({});
  const [batchExtraModels, setBatchExtraModels] = useState<Record<string, string[]>>({});
  const [activeBatch, setActiveBatch] = useState<EndpointProbeBatchDetail | null>(null);
  const [selectedBatchId, setSelectedBatchId] = useState<string | null>(null);
  const [expandedRunId, setExpandedRunId] = useState<string | null>(null);
  const [runDetails, setRunDetails] = useState<Record<string, EndpointProbeRunDetail>>({});
  const [loadingRunId, setLoadingRunId] = useState<string | null>(null);
  const [runDetailError, setRunDetailError] = useState<string | null>(null);
  const [historyPage, setHistoryPage] = useState(1);
  const [historyPageSize, setHistoryPageSize] = useState(20);
  const [historyStatus, setHistoryStatus] = useState("all");
  const [historyKeyword, setHistoryKeyword] = useState("");
  const [promotionRun, setPromotionRun] = useState<EndpointProbeRunDetail | null>(null);
  const submittedTemporaryKeys = useRef(new Map<string, string>());
  const providerState = useEndpointProbeProviders(batchExtraModels);
  const probeEvents = useEndpointProbeEvents({
    activeBatchId: activeBatch?.id ?? null,
    queryClient,
    setActiveBatch,
    setRunDetails,
  });

  const historyQuery = useQuery({
    queryKey: queryKeys.endpointProbeBatches(
      historyPage,
      historyPageSize,
      historyStatus,
      historyKeyword,
    ),
    queryFn: () =>
      api.listEndpointProbeBatchesPage({
        page: historyPage,
        page_size: historyPageSize,
        status: historyStatus === "all" ? undefined : historyStatus,
        keyword: historyKeyword.trim() || undefined,
      }),
  });

  const batchDetailQuery = useQuery({
    queryKey: queryKeys.endpointProbeBatchDetail(selectedBatchId ?? ""),
    queryFn: () => api.getEndpointProbeBatchDetail(selectedBatchId ?? ""),
    enabled: Boolean(selectedBatchId),
  });
  useEffect(() => {
    if (!batchDetailQuery.data) return;
    setActiveBatch(batchDetailQuery.data);
  }, [batchDetailQuery.data]);

  const startMutation = useMutation({
    mutationFn: (input: EndpointProbeStartInput) => api.startEndpointProbe(input),
    onSuccess: async (batch, input) => {
      const detail = await api.getEndpointProbeBatchDetail(batch.id);
      setSelectedBatchId(batch.id);
      setActiveBatch(detail);
      setExpandedRunId(detail.runs[0]?.id ?? null);
      if (input.targets.length === 1 && input.targets[0].source === "temporary") {
        const runId = detail.runs[0]?.id;
        if (runId) submittedTemporaryKeys.current.set(runId, temporary.api_key);
      }
      await invalidateEndpointProbeQueries(queryClient);
      notify({
        title: detail.total_runs > 1 ? "批量测活已启动" : "站点测活已启动",
        description: `正在执行 ${detail.total_runs} 个模型请求。`,
        tone: "info",
      });
    },
    onError: (error) => {
      notify({
        title: "测活无法启动",
        description: errorMessage(error),
        tone: "danger",
      });
    },
  });

  const stopMutation = useMutation({
    mutationFn: api.stopEndpointProbe,
    onSuccess: (result) => {
      showToast({
        message: result.stopped ? "正在停止测活" : "该批次已经结束",
        tone: result.stopped ? "warning" : "info",
      });
    },
  });

  const scanMutation = useMutation({
    mutationFn: (input: EndpointProbeModelScanInput) =>
      api.scanEndpointProbeModels(input),
    onSuccess: async (result, input) => {
      if (input.source === "temporary") {
        setTemporaryModels(result.models);
        setTemporary((current) => ({
          ...current,
          model: result.models.some((model) => model.name === current.model)
            ? current.model
            : result.models[0]?.name ?? current.model,
        }));
      } else {
        await queryClient.invalidateQueries({
          queryKey: queryKeys.providerModels(input.provider_id),
        });
        await queryClient.invalidateQueries({ queryKey: queryKeys.providers() });
      }
      notify({
        title: result.models.length ? "模型列表已更新" : "模型列表为空",
        description: result.message,
        tone: result.models.length ? "success" : "info",
      });
    },
    onError: (error) => {
      notify({
        title: "模型列表获取失败",
        description: errorMessage(error),
        tone: "danger",
      });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: api.deleteEndpointProbeBatch,
    onSuccess: async (_, batchId) => {
      if (selectedBatchId === batchId) {
        setSelectedBatchId(null);
        setActiveBatch(null);
      }
      await invalidateEndpointProbeQueries(queryClient);
      showToast({ message: "测活批次已删除", tone: "success" });
    },
  });

  const promotionMutation = useMutation({
    mutationFn: api.promoteEndpointProbeTarget,
    onSuccess: async (result, input) => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.providers() });
      submittedTemporaryKeys.current.delete(input.run_id);
      setPromotionRun(null);
      notify({
        title: result.status === "already_exists" ? "服务商已经存在" : "已保存为服务商",
        description: result.warning ?? `已保存 ${result.provider.name}`,
        tone: result.warning ? "warning" : "success",
      });
    },
    onError: (error) => {
      notify({
        title: "保存服务商失败",
        description: errorMessage(error),
        tone: "danger",
      });
    },
  });

  const importMutation = useMutation({
    mutationFn: (items: ProviderImportItem[]) => api.importProviders({ items }),
    onSuccess: async (result) => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.providers() });
      notify({
        title: `已导入 ${result.created} 个服务商`,
        description: `跳过 ${result.skipped} 个，失败 ${result.failed} 个。`,
        tone: result.failed ? "warning" : "success",
      });
    },
    onError: (error) => {
      notify({
        title: "服务商导入失败",
        description: errorMessage(error),
        tone: "danger",
      });
    },
  });

  const selectedRunCount = useMemo(() => countSelectedProbeRuns(batchModels), [batchModels]);
  const formSnapshot = {
    workspaceMode,
    singleSource,
    common,
    temporary,
    singleProviderId: providerState.singleProviderId,
    singleProviderModel: providerState.singleProviderModel,
    batchModels,
  };
  const startIssue = validateEndpointProbeStart(formSnapshot, probeEvents.listenersReady);

  const start = () => {
    if (startIssue) {
      notify({ title: "请完善测活配置", description: startIssue, tone: "danger" });
      return;
    }
    probeEvents.resetStreams();
    submittedTemporaryKeys.current.clear();
    setRunDetails({});
    setExpandedRunId(null);
    setRunDetailError(null);
    startMutation.mutate(buildEndpointProbeStartInput(formSnapshot));
  };

  const selectBatch = (batchId: string) => {
    setSelectedBatchId(batchId);
    setActiveBatch(null);
    setExpandedRunId(null);
    setRunDetails({});
    probeEvents.resetStreams();
    setRunDetailError(null);
  };

  const expandRun = async (runId: string | null) => {
    setExpandedRunId(runId);
    setRunDetailError(null);
    if (!runId || runDetails[runId]) return;
    const summary = activeBatch?.runs.find((run) => run.id === runId);
    if (summary?.status === "running" || summary?.status === "pending") return;
    setLoadingRunId(runId);
    try {
      const detail = await api.getEndpointProbeRunDetail(runId);
      setRunDetails((current) => ({ ...current, [runId]: detail }));
    } catch (error) {
      setRunDetailError(errorMessage(error));
    } finally {
      setLoadingRunId(null);
    }
  };

  const toggleBatchModel = (providerId: string, model: string, checked: boolean) => {
    setBatchModels((current) => {
      const models = current[providerId] ?? [];
      const nextModels = checked
        ? Array.from(new Set([...models, model]))
        : models.filter((value) => value !== model);
      const next = { ...current };
      if (nextModels.length) next[providerId] = nextModels;
      else delete next[providerId];
      return next;
    });
  };

  const addManualProviderModel = (providerId: string, model: string) => {
    const normalized = model.trim();
    if (!normalized) return;
    setBatchExtraModels((current) => ({
      ...current,
      [providerId]: Array.from(new Set([...(current[providerId] ?? []), normalized])),
    }));
    toggleBatchModel(providerId, normalized, true);
  };

  const openPromotion = async (runId: string) => {
    const existing = runDetails[runId];
    if (existing) {
      setPromotionRun(existing);
      return;
    }
    const detail = await api.getEndpointProbeRunDetail(runId);
    setRunDetails((current) => ({ ...current, [runId]: detail }));
    setPromotionRun(detail);
  };

  return {
    activeBatch,
    addManualProviderModel,
    batchDetailError: batchDetailQuery.error,
    batchDetailLoading: batchDetailQuery.isFetching,
    batchModels,
    common,
    deleteBatch: (batchId: string) => deleteMutation.mutate(batchId),
    deletingBatchId: deleteMutation.isPending ? deleteMutation.variables : null,
    expandRun,
    expandedRunId,
    history: historyQuery.data,
    historyError: historyQuery.error,
    historyKeyword,
    historyLoading: historyQuery.isFetching,
    historyPage,
    historyPageSize,
    historyStatus,
    importPending: importMutation.isPending,
    importProviders: (items: ProviderImportItem[]) => importMutation.mutateAsync(items),
    importResult: importMutation.data as ProviderImportResult | undefined,
    listenersReady: probeEvents.listenersReady,
    loadingRunId,
    openPromotion,
    promotionDefaultKey: promotionRun
      ? submittedTemporaryKeys.current.get(promotionRun.id) ?? ""
      : "",
    promotionError: promotionMutation.error,
    promotionPending: promotionMutation.isPending,
    promotionRun,
    providerModels: providerState.providerModels,
    providers: providerState.providers,
    providersLoading: providerState.providersLoading,
    refreshProviderModels: (providerId: string) =>
      {
        providerState.ensureProviderModels(providerId);
        scanMutation.mutate({ source: "provider", provider_id: providerId });
      },
    ensureProviderModels: providerState.ensureProviderModels,
    runDetailError,
    runDetails,
    running: startMutation.isPending || activeBatch?.status === "running",
    scanningProviderId:
      scanMutation.isPending && scanMutation.variables?.source === "provider"
        ? scanMutation.variables.provider_id
        : null,
    scanningTemporary:
      scanMutation.isPending && scanMutation.variables?.source === "temporary",
    selectBatch,
    selectedBatchId,
    setBatchModels,
    setCommon,
    setHistoryKeyword: (value: string) => {
      setHistoryKeyword(value);
      setHistoryPage(1);
    },
    setHistoryPage,
    setHistoryPageSize: (value: number) => {
      setHistoryPageSize(value);
      setHistoryPage(1);
    },
    setHistoryStatus: (value: string) => {
      setHistoryStatus(value);
      setHistoryPage(1);
    },
    setPromotionRun,
    setSingleProviderId: providerState.setSingleProviderId,
    setSingleProviderModel: providerState.setSingleProviderModel,
    setSingleSource,
    setTemporary,
    setWorkspaceMode,
    singleProviderId: providerState.singleProviderId,
    singleProviderModel: providerState.singleProviderModel,
    singleProviderModels: providerState.singleProviderModels,
    singleSource,
    start,
    startIssue,
    stop: () => activeBatch && stopMutation.mutate(activeBatch.id),
    stopping: stopMutation.isPending,
    streamText: probeEvents.streamText,
    submitPromotion: (name: string, apiKey: string, syncModels: boolean) => {
      if (!promotionRun) return;
      promotionMutation.mutate({
        run_id: promotionRun.id,
        name: name.trim() || undefined,
        api_key: apiKey,
        sync_models: syncModels,
      });
    },
    temporary,
    temporaryModels,
    toggleBatchModel,
    selectedRunCount,
    workspaceMode,
    scanTemporaryModels: () =>
      scanMutation.mutate({
        source: "temporary",
        base_url: temporary.base_url.trim(),
        api_key: temporary.api_key,
        interface_type: temporary.interface_type,
      }),
    resetTemporaryModels: () => setTemporaryModels([]),
  };
}

async function invalidateEndpointProbeQueries(
  queryClient: ReturnType<typeof useQueryClient>,
) {
  await Promise.all([
    queryClient.invalidateQueries({ queryKey: ["endpoint-probe-batches"] }),
    queryClient.invalidateQueries({ queryKey: ["endpoint-probe-batch-detail"] }),
    queryClient.invalidateQueries({ queryKey: queryKeys.providers() }),
  ]);
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
