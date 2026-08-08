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
  AppConfig,
  EndpointProbeBatchDetail,
  EndpointProbeModelOption,
  EndpointProbePromptTemplatesConfig,
  EndpointProbeModelScanInput,
  EndpointProbeRunDetail,
  EndpointProbeRunSummary,
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
import {
  DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATES,
  createEndpointProbePromptTemplate,
  normalizeEndpointProbePromptTemplates,
  selectedEndpointProbePromptTemplate,
} from "../domain/endpointProbePromptTemplates";
import {
  canPromoteEndpointProbeRun,
  pickDefaultEndpointProbeRunId,
} from "../domain/endpointProbePresentation";
import { useEndpointProbeEvents } from "./useEndpointProbeEvents";
import { useEndpointProbeProviders } from "./useEndpointProbeProviders";

export function useEndpointProbeView() {
  const queryClient = useQueryClient();
  const { notify } = useNotification();
  const { showToast } = useToast();
  const [workspaceMode, setWorkspaceMode] =
    useState<EndpointProbeWorkspaceMode>("batch");
  const [singleSource, setSingleSource] =
    useState<EndpointProbeSingleSource>("provider");
  const [common, setCommon] = useState(createEndpointProbeCommonForm);
  const [promptTemplatesConfig, setPromptTemplatesConfig] =
    useState<EndpointProbePromptTemplatesConfig>(
      DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATES,
    );
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
  const promptTemplatesInitialized = useRef(false);
  const autoExpandedBatchId = useRef<string | null>(null);
  const providerState = useEndpointProbeProviders(batchExtraModels);
  const probeEvents = useEndpointProbeEvents({
    activeBatchId: activeBatch?.id ?? null,
    queryClient,
    setActiveBatch,
    setRunDetails,
  });

  const configQuery = useQuery({
    queryKey: queryKeys.appConfig(),
    queryFn: api.getAppConfig,
  });

  useEffect(() => {
    if (!configQuery.data || promptTemplatesInitialized.current) return;
    const templatesConfig = normalizeEndpointProbePromptTemplates(
      configQuery.data.endpoint_probe_prompt_templates,
    );
    const selected = selectedEndpointProbePromptTemplate(templatesConfig);
    setPromptTemplatesConfig(templatesConfig);
    setCommon((current) => ({ ...current, prompt: selected.prompt }));
    promptTemplatesInitialized.current = true;
  }, [configQuery.data]);

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
    hydrateBatchDetail(batchDetailQuery.data);
  }, [batchDetailQuery.data]);

  const startMutation = useMutation({
    mutationFn: (input: EndpointProbeStartInput) => api.startEndpointProbe(input),
    onSuccess: async (batch, input) => {
      const detail = await api.getEndpointProbeBatchDetail(batch.id);
      setSelectedBatchId(batch.id);
      hydrateBatchDetail(detail);
      if (input.targets.length === 1 && input.targets[0].source === "temporary") {
        const runId = detail.runs[0]?.id;
        if (runId) submittedTemporaryKeys.current.set(runId, temporary.api_key);
      }
      await invalidateEndpointProbeQueries(queryClient);
      notify({
        title: "站点测活已启动",
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

  const savePromptTemplatesMutation = useMutation({
    mutationFn: async ({
      templatesConfig,
    }: {
      templatesConfig: EndpointProbePromptTemplatesConfig;
      message: string;
    }) => {
      const config = configQuery.data;
      if (!config) throw new Error("Prompt 模板配置仍在加载，请稍候。");
      return api.updateAppConfig({
        ...config,
        endpoint_probe_prompt_templates:
          normalizeEndpointProbePromptTemplates(templatesConfig),
      });
    },
    onSuccess: async (result, variables) => {
      const templatesConfig = normalizeEndpointProbePromptTemplates(
        result.config.endpoint_probe_prompt_templates,
      );
      setPromptTemplatesConfig(templatesConfig);
      queryClient.setQueryData<AppConfig>(queryKeys.appConfig(), result.config);
      await queryClient.invalidateQueries({ queryKey: queryKeys.appConfig() });
      showToast({ message: variables.message, tone: "success" });
    },
    onError: (error) => {
      notify({
        title: "Prompt 模板保存失败",
        description: errorMessage(error),
        tone: "danger",
      });
    },
  });

  const selectedRunCount = useMemo(() => countSelectedProbeRuns(batchModels), [batchModels]);
  const promotableRun = useMemo(
    () =>
      activeBatch?.runs.length === 1
        ? activeBatch.runs.find(canPromoteEndpointProbeRun) ?? null
        : null,
    [activeBatch],
  );
  const selectedPromptTemplate = useMemo(
    () => selectedEndpointProbePromptTemplate(promptTemplatesConfig),
    [promptTemplatesConfig],
  );
  const promptTemplateDirty = selectedPromptTemplate.prompt !== common.prompt;
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
    autoExpandedBatchId.current = null;
    probeEvents.resetStreams();
    submittedTemporaryKeys.current.clear();
    setRunDetails({});
    setExpandedRunId(null);
    setRunDetailError(null);
    startMutation.mutate(buildEndpointProbeStartInput(formSnapshot));
  };

  const selectBatch = (batchId: string) => {
    if (batchId === selectedBatchId) return;
    autoExpandedBatchId.current = null;
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
    await loadRunDetail(runId);
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

  const selectPromptTemplate = (templateId: string) => {
    const nextConfig = normalizeEndpointProbePromptTemplates({
      ...promptTemplatesConfig,
      selected_id: templateId,
    });
    const selected = selectedEndpointProbePromptTemplate(nextConfig);
    setPromptTemplatesConfig(nextConfig);
    setCommon((current) => ({ ...current, prompt: selected.prompt }));
  };

  const saveCurrentPromptTemplate = () => {
    const prompt = common.prompt;
    if (!prompt.trim()) {
      notify({
        title: "Prompt 模板无法保存",
        description: "模板内容不能为空。",
        tone: "danger",
      });
      return;
    }
    const nextConfig = normalizeEndpointProbePromptTemplates({
      selected_id: promptTemplatesConfig.selected_id,
      items: promptTemplatesConfig.items.map((item) =>
        item.id === promptTemplatesConfig.selected_id ? { ...item, prompt } : item,
      ),
    });
    setPromptTemplatesConfig(nextConfig);
    savePromptTemplatesMutation.mutate({
      templatesConfig: nextConfig,
      message: "Prompt 模板已保存",
    });
  };

  const addPromptTemplate = () => {
    const template = createEndpointProbePromptTemplate(
      promptTemplatesConfig.items,
      common.prompt,
    );
    const nextConfig = normalizeEndpointProbePromptTemplates({
      selected_id: template.id,
      items: [...promptTemplatesConfig.items, template],
    });
    setPromptTemplatesConfig(nextConfig);
    setCommon((current) => ({ ...current, prompt: template.prompt }));
    savePromptTemplatesMutation.mutate({
      templatesConfig: nextConfig,
      message: "Prompt 模板已新增",
    });
  };

  function hydrateBatchDetail(detail: EndpointProbeBatchDetail) {
    setActiveBatch(detail);
    if (autoExpandedBatchId.current === detail.id) return;
    const runId = pickDefaultEndpointProbeRunId(detail.runs);
    autoExpandedBatchId.current = detail.id;
    setExpandedRunId(runId);
    if (!runId) return;
    void loadRunDetail(runId, detail.runs.find((run) => run.id === runId));
  }

  async function loadRunDetail(
    runId: string | null,
    summary?: EndpointProbeRunSummary,
  ) {
    if (!runId || runDetails[runId]) return;
    const runSummary = summary ?? activeBatch?.runs.find((run) => run.id === runId);
    if (runSummary?.status === "running" || runSummary?.status === "pending") return;
    setLoadingRunId(runId);
    try {
      const detail = await api.getEndpointProbeRunDetail(runId);
      setRunDetails((current) => ({ ...current, [runId]: detail }));
    } catch (error) {
      setRunDetailError(errorMessage(error));
    } finally {
      setLoadingRunId(null);
    }
  }

  async function copyProbeText(label: string, value?: string | null) {
    if (!value) {
      notify({
        title: "没有可复制内容",
        description: "当前只保留了摘要，完整正文需要在测活前开启保存。",
        tone: "warning",
      });
      return;
    }
    try {
      await navigator.clipboard.writeText(value);
      showToast({ message: `${label}已复制`, tone: "success" });
    } catch {
      notify({
        title: `${label}复制失败`,
        description: "系统剪贴板暂不可用，请稍后重试。",
        tone: "danger",
      });
    }
  }

  return {
    activeBatch,
    addManualProviderModel,
    addPromptTemplate,
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
    promptTemplateDirty,
    promptTemplates: promptTemplatesConfig.items,
    promptTemplatesLoading: configQuery.isFetching && !promptTemplatesInitialized.current,
    promotableRun,
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
    saveCurrentPromptTemplate,
    savingPromptTemplate: savePromptTemplatesMutation.isPending,
    selectPromptTemplate,
    selectedPromptTemplateId: promptTemplatesConfig.selected_id,
    resetTemporaryModels: () => setTemporaryModels([]),
    copyProbeText,
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
