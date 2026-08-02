import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import { useToast } from "../../../components/ui/Toast";
import type {
  SiteProbeInterfaceType,
  SiteProbeModelOption,
  SiteProbeModelScanInput,
  SiteProbeRunDetail,
  SiteProbeRunInput,
} from "../../../types/api";

export type SiteProbeFormState = {
  name: string;
  base_url: string;
  api_key: string;
  interface_type: SiteProbeInterfaceType;
  model: string;
  prompt: string;
  streaming: boolean;
  max_output_tokens: number;
  timeout_seconds: number;
  save_body: boolean;
};

export const defaultSiteProbeForm = (): SiteProbeFormState => ({
  name: "",
  base_url: "",
  api_key: "",
  interface_type: "OpenAI",
  model: "",
  prompt: "请用一句话回复：MyLLMBenchmark 测活成功。",
  streaming: true,
  max_output_tokens: 256,
  timeout_seconds: 60,
  save_body: false,
});

export function useSiteProbeView() {
  const queryClient = useQueryClient();
  const { pushToast } = useToast();
  const [form, setForm] = useState<SiteProbeFormState>(defaultSiteProbeForm);
  const [activeRun, setActiveRun] = useState<SiteProbeRunDetail | null>(null);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [statusFilter, setStatusFilter] = useState("all");
  const [keyword, setKeyword] = useState("");
  const [modelOptions, setModelOptions] = useState<SiteProbeModelOption[]>([]);
  const [modelScanMessage, setModelScanMessage] = useState<string | null>(null);
  const [manualModelEntry, setManualModelEntry] = useState(false);

  const historyQuery = useQuery({
    queryKey: queryKeys.siteProbeRuns(page, pageSize, statusFilter, keyword),
    queryFn: () =>
      api.listSiteProbeRunsPage({
        page,
        page_size: pageSize,
        status: statusFilter === "all" ? undefined : statusFilter,
        keyword: keyword.trim() || undefined,
      }),
  });

  const detailQuery = useQuery({
    queryKey: queryKeys.siteProbeRunDetail(selectedRunId ?? ""),
    queryFn: () => api.getSiteProbeRunDetail(selectedRunId ?? ""),
    enabled: Boolean(selectedRunId),
  });

  const runMutation = useMutation({
    mutationFn: (input: SiteProbeRunInput) => api.runSiteProbe(input),
    onSuccess: async (detail) => {
      setSelectedRunId(null);
      setActiveRun(detail);
      await queryClient.invalidateQueries({ queryKey: ["site-probe-runs"] });
      pushToast({
        title: detail.status === "passed" ? "站点测活通过" : "站点测活失败",
        description:
          detail.status === "passed"
            ? `模型 ${detail.model} 已返回响应，耗时 ${detail.latency_ms}ms。`
            : detail.error_message ?? "请求失败，请检查 Base URL、Key、模型名称和网关日志。",
        tone: detail.status === "passed" ? "success" : "danger",
      });
    },
    onError: (error) => {
      pushToast({
        title: "站点测活无法启动",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const modelScanMutation = useMutation({
    mutationFn: (input: SiteProbeModelScanInput) => api.scanSiteProbeModels(input),
    onSuccess: (result) => {
      setModelOptions(result.models);
      setModelScanMessage(result.message);
      setManualModelEntry(result.models.length === 0);
      setForm((current) => ({
        ...current,
        model: result.models.some((item) => item.name === current.model)
          ? current.model
          : result.models[0]?.name ?? current.model,
      }));
      pushToast({
        title: result.models.length > 0 ? "模型列表已获取" : "模型列表为空",
        description: result.message,
        tone: result.models.length > 0 ? "success" : "info",
      });
    },
    onError: (error) => {
      pushToast({
        title: "模型列表获取失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: api.deleteSiteProbeRun,
    onSuccess: async (_, runId) => {
      if (selectedRunId === runId) setSelectedRunId(null);
      if (activeRun?.id === runId) setActiveRun(null);
      await queryClient.invalidateQueries({ queryKey: ["site-probe-runs"] });
      pushToast({
        title: "测活记录已删除",
        tone: "success",
      });
    },
  });

  const visibleRun = useMemo(
    () => detailQuery.data ?? activeRun,
    [activeRun, detailQuery.data],
  );

  const submit = () => {
    runMutation.mutate({
      name: form.name.trim() || undefined,
      base_url: form.base_url.trim(),
      api_key: form.api_key,
      interface_type: form.interface_type,
      model: form.model.trim(),
      prompt: form.prompt,
      streaming: form.streaming,
      max_output_tokens: Number(form.max_output_tokens),
      timeout_seconds: Number(form.timeout_seconds),
      save_body: form.save_body,
    });
  };

  const resetModelScan = () => {
    modelScanMutation.reset();
    setModelOptions([]);
    setModelScanMessage(null);
  };

  const scanModels = () => {
    modelScanMutation.mutate({
      base_url: form.base_url.trim(),
      api_key: form.api_key,
      interface_type: form.interface_type,
    });
  };

  return {
    activeRun: visibleRun,
    deleteRun: (runId: string) => deleteMutation.mutate(runId),
    deletingRunId: deleteMutation.isPending ? deleteMutation.variables : null,
    detailError: detailQuery.error,
    detailLoading: detailQuery.isFetching,
    form,
    history: historyQuery.data,
    historyError: historyQuery.error,
    historyLoading: historyQuery.isFetching,
    keyword,
    manualModelEntry,
    modelOptions,
    modelScanError: modelScanMutation.error,
    modelScanMessage,
    page,
    pageSize,
    running: runMutation.isPending,
    scanningModels: modelScanMutation.isPending,
    scanModels,
    selectedRunId,
    setForm,
    setManualModelEntry,
    setKeyword: (value: string) => {
      setKeyword(value);
      setPage(1);
    },
    setPage,
    setPageSize: (value: number) => {
      setPageSize(value);
      setPage(1);
    },
    setSelectedRunId: (runId: string) => {
      setSelectedRunId(runId);
      setActiveRun(null);
    },
    setStatusFilter: (value: string) => {
      setStatusFilter(value);
      setPage(1);
    },
    resetModelScan,
    statusFilter,
    submit,
  };
}
