import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import { useNotification } from "../../../components/ui/Notification";
import type {
  DatasetAppendInput,
  DatasetExportInput,
  DatasetImportInput,
  DatasetSampleBatchDeleteInput,
  DatasetSampleCreateInput,
  DatasetSampleUpdateInput,
  DatasetSummary,
  DatasetUpdateInput,
  DatasetValidationResult,
} from "../../../types/api";
import type { DatasetTypeFilter } from "../constants";
import { useDatasetsCatalog } from "./useDatasetsCatalog";

type DetailMode = "view" | "edit";

const DEFAULT_SAMPLE_PAGE = 1;
const DEFAULT_SAMPLE_PAGE_SIZE = 50;

export function useDatasetsController() {
  const queryClient = useQueryClient();
  const { notify } = useNotification();
  const [importOpen, setImportOpen] = useState(false);
  const [selectedDatasetId, setSelectedDatasetId] = useState<string | null>(null);
  const [detailMode, setDetailMode] = useState<DetailMode>("view");
  const [deleteTarget, setDeleteTarget] = useState<DatasetSummary | null>(null);
  const [sampleKeywordInput, setSampleKeywordInput] = useState("");
  const [sampleKeyword, setSampleKeyword] = useState("");
  const [samplePage, setSamplePage] = useState(DEFAULT_SAMPLE_PAGE);
  const [samplePageSize, setSamplePageSize] = useState(DEFAULT_SAMPLE_PAGE_SIZE);
  const [validationResult, setValidationResult] =
    useState<DatasetValidationResult | null>(null);
  const { activeType, datasets, filtered, setActiveType } = useDatasetsCatalog();

  const selectedDataset = useMemo(
    () => datasets.find((dataset) => dataset.id === selectedDatasetId) ?? null,
    [datasets, selectedDatasetId],
  );

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setSampleKeyword(sampleKeywordInput.trim());
    }, 300);

    return () => window.clearTimeout(timer);
  }, [sampleKeywordInput]);

  const samplesQuery = useQuery({
    enabled: Boolean(selectedDatasetId),
    queryKey: selectedDatasetId
      ? queryKeys.datasetSamples(
          selectedDatasetId,
          samplePage,
          samplePageSize,
          sampleKeyword,
        )
      : ["dataset-samples", "idle"],
    queryFn: () =>
      api.listDatasetSamplesPage({
        dataset_id: selectedDatasetId!,
        page: samplePage,
        page_size: samplePageSize,
        keyword: sampleKeyword || null,
      }),
  });

  const samplePageData = samplesQuery.data ?? {
    items: [],
    total: 0,
    page: samplePage,
    page_size: samplePageSize,
  };
  const visibleSamplePageData = {
    ...samplePageData,
    page: samplePage,
    page_size: samplePageSize,
  };

  const invalidateDatasets = async () => {
    await queryClient.invalidateQueries({ queryKey: queryKeys.datasets() });
  };

  const invalidateSamples = async (datasetId: string) => {
    await queryClient.invalidateQueries({
      queryKey: queryKeys.datasetSamplesRoot(datasetId),
    });
  };

  const resetSamplePaging = () => {
    setSampleKeywordInput("");
    setSampleKeyword("");
    setSamplePage(DEFAULT_SAMPLE_PAGE);
    setSamplePageSize(DEFAULT_SAMPLE_PAGE_SIZE);
  };

  const importMutation = useMutation({
    mutationFn: api.importDataset,
    onSuccess: async (dataset) => {
      await invalidateDatasets();
      setImportOpen(false);
      setSelectedDatasetId(dataset.id);
      setDetailMode("view");
      resetSamplePaging();
      notify({
        title: "数据集导入成功",
        description: `${dataset.name} / ${dataset.sample_count.toLocaleString("zh-CN")} 条样本`,
        tone: "success",
      });
    },
    onError: (error) => {
      notify({
        title: "数据集导入失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const updateDatasetMutation = useMutation({
    mutationFn: api.updateDataset,
    onSuccess: async (dataset) => {
      await invalidateDatasets();
      setSelectedDatasetId(dataset.id);
      setDetailMode("view");
      notify({
        title: "数据集已更新",
        description: dataset.name,
        tone: "success",
      });
    },
    onError: (error) => {
      notify({
        title: "数据集更新失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const deleteDatasetMutation = useMutation({
    mutationFn: api.deleteDataset,
    onSuccess: async (result) => {
      await invalidateDatasets();
      if (result.deleted && selectedDatasetId === result.id) {
        closeDetail();
      }
      setDeleteTarget(null);
      notify({
        title: result.deleted ? "数据集已删除" : "数据集未删除",
        description: result.deleted
          ? "样本正文已清理，历史任务和报告仍可读取数据集名称。"
          : "没有找到需要删除的数据集。",
        tone: result.deleted ? "success" : "danger",
      });
    },
    onError: (error) => {
      notify({
        title: "数据集删除失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const createSampleMutation = useMutation({
    mutationFn: api.createDatasetSample,
    onSuccess: async (sample) => {
      await invalidateDatasets();
      if (selectedDatasetId) {
        const currentTotal = selectedDataset?.sample_count ?? samplePageData.total;
        const nextTotal = Math.max(currentTotal + 1, sample.sample_index + 1);
        setSampleKeywordInput("");
        setSampleKeyword("");
        setSamplePage(Math.max(1, Math.ceil(nextTotal / samplePageSize)));
        await invalidateSamples(selectedDatasetId);
      }
      notify({
        title: "样本已新增",
        description: `第 ${sample.sample_index + 1} 条 Prompt`,
        tone: "success",
      });
    },
    onError: (error) => {
      notify({
        title: "样本新增失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const updateSampleMutation = useMutation({
    mutationFn: api.updateDatasetSample,
    onSuccess: async (sample) => {
      await invalidateDatasets();
      if (selectedDatasetId) {
        await invalidateSamples(selectedDatasetId);
      }
      notify({
        title: "样本已更新",
        description: `第 ${sample.sample_index + 1} 条 Prompt`,
        tone: "success",
      });
    },
    onError: (error) => {
      notify({
        title: "样本更新失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const deleteSampleMutation = useMutation({
    mutationFn: api.deleteDatasetSample,
    onSuccess: async (result) => {
      await invalidateDatasets();
      if (selectedDatasetId) {
        const deletingLastVisibleItem =
          result.deleted && samplePageData.items.length <= 1 && samplePage > 1;
        if (deletingLastVisibleItem) {
          setSamplePage((page) => Math.max(1, page - 1));
        }
        await invalidateSamples(selectedDatasetId);
      }
      notify({
        title: result.deleted ? "样本已删除" : "样本未删除",
        description: result.deleted
          ? "样本统计已重新计算。"
          : "没有找到需要删除的样本。",
        tone: result.deleted ? "success" : "danger",
      });
    },
    onError: (error) => {
      notify({
        title: "样本删除失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const appendSamplesMutation = useMutation({
    mutationFn: api.appendDatasetSamples,
    onSuccess: async (dataset) => {
      await invalidateDatasets();
      await invalidateSamples(dataset.id);
      setSelectedDatasetId(dataset.id);
      setSampleKeywordInput("");
      setSampleKeyword("");
      setSamplePage(Math.max(1, Math.ceil(dataset.sample_count / samplePageSize)));
      setValidationResult(null);
      notify({
        title: "样本已追加",
        description: `${dataset.name} 当前 ${dataset.sample_count.toLocaleString("zh-CN")} 条样本`,
        tone: "success",
      });
    },
    onError: (error) => {
      notify({
        title: "追加样本失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const batchDeleteMutation = useMutation({
    mutationFn: api.deleteDatasetSamplesBatch,
    onSuccess: async (result) => {
      await invalidateDatasets();
      if (selectedDatasetId) {
        const deletingLastPage =
          result.deleted && samplePageData.items.length <= 1 && samplePage > 1;
        if (deletingLastPage) {
          setSamplePage((page) => Math.max(1, page - 1));
        }
        await invalidateSamples(selectedDatasetId);
      }
      setValidationResult(null);
      notify({
        title: result.deleted ? "已批量删除样本" : "未删除样本",
        description: result.deleted ? "当前页样本统计已刷新。" : "没有匹配到要删除的样本。",
        tone: result.deleted ? "success" : "danger",
      });
    },
    onError: (error) => {
      notify({
        title: "批量删除失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const exportDatasetMutation = useMutation({
    mutationFn: api.exportDataset,
    onSuccess: (result) => {
      notify({
        title: "数据集已导出",
        description: result.file_path,
        tone: "success",
      });
    },
    onError: (error) => {
      notify({
        title: "数据集导出失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  const validateDatasetMutation = useMutation({
    mutationFn: api.validateDatasetSamples,
    onSuccess: (result) => {
      setValidationResult(result);
      notify({
        title: result.status === "passed" ? "质量检查通过" : "质量检查完成",
        description:
          result.issues.length > 0
            ? `发现 ${result.issues.length} 类问题`
            : "未发现明显样本问题",
        tone: result.status === "passed" ? "success" : "danger",
      });
    },
    onError: (error) => {
      notify({
        title: "质量检查失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger",
      });
    },
  });

  function openDetail(dataset: DatasetSummary, mode: DetailMode = "view") {
    setSelectedDatasetId(dataset.id);
    setDetailMode(mode);
    setValidationResult(null);
    resetSamplePaging();
  }

  function closeDetail() {
    setSelectedDatasetId(null);
    setDetailMode("view");
    setValidationResult(null);
    resetSamplePaging();
  }

  function updateSampleKeyword(value: string) {
    setSampleKeywordInput(value);
    setSamplePage(DEFAULT_SAMPLE_PAGE);
  }

  function updateSamplePageSize(value: number) {
    setSamplePageSize(value);
    setSamplePage(DEFAULT_SAMPLE_PAGE);
  }

  return {
    activeType,
    createSample: (input: DatasetSampleCreateInput) =>
      createSampleMutation.mutateAsync(input),
    createSamplePending: createSampleMutation.isPending,
    appendSamples: (input: DatasetAppendInput) =>
      appendSamplesMutation.mutateAsync(input),
    appendSamplesPending: appendSamplesMutation.isPending,
    batchDeleteSamples: (input: DatasetSampleBatchDeleteInput) =>
      batchDeleteMutation.mutate(input),
    batchDeletePending: batchDeleteMutation.isPending,
    deleteDataset: (datasetId: string) => deleteDatasetMutation.mutate(datasetId),
    deleteDatasetPending: deleteDatasetMutation.isPending,
    deleteSample: (sampleId: string) => deleteSampleMutation.mutate(sampleId),
    deleteSamplePending: deleteSampleMutation.isPending,
    deleteTarget,
    detailMode,
    filtered,
    importDataset: (input: DatasetImportInput) => importMutation.mutateAsync(input),
    exportDataset: (input: DatasetExportInput) => exportDatasetMutation.mutate(input),
    exportDatasetPending: exportDatasetMutation.isPending,
    validateDataset: (datasetId: string) => validateDatasetMutation.mutate(datasetId),
    validateDatasetPending: validateDatasetMutation.isPending,
    importOpen,
    importPending: importMutation.isPending,
    openDetail,
    sampleKeyword: sampleKeywordInput,
    samplePage,
    samplePageData: visibleSamplePageData,
    samplePageSize,
    samplesLoading: samplesQuery.isLoading,
    samplesFetching: samplesQuery.isFetching,
    selectedDataset,
    setActiveType: (value: DatasetTypeFilter) => {
      setActiveType(value);
      closeDetail();
    },
    setDeleteTarget,
    setDetailMode,
    setImportOpen,
    setSampleKeyword: updateSampleKeyword,
    setSamplePage,
    setSamplePageSize: updateSamplePageSize,
    updateDataset: (input: DatasetUpdateInput) =>
      updateDatasetMutation.mutateAsync(input),
    updateDatasetPending: updateDatasetMutation.isPending,
    updateSample: (input: DatasetSampleUpdateInput) =>
      updateSampleMutation.mutateAsync(input),
    updateSamplePending: updateSampleMutation.isPending,
    validationResult,
    closeDetail,
  };
}
