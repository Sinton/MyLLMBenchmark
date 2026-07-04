import { useEffect, useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import type { ChartMetric } from "../types";

export function useReportsView() {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [chartMetric, setChartMetric] = useState<ChartMetric>("latency");
  const { data: reports = [] } = useQuery({
    queryKey: queryKeys.reports(),
    queryFn: api.listReports,
  });

  useEffect(() => {
    if (!selectedId && reports.length > 0) {
      setSelectedId(reports[0].id);
    }
  }, [reports, selectedId]);

  const selectedReport = useMemo(
    () => reports.find((report) => report.id === selectedId) ?? reports[0],
    [reports, selectedId],
  );

  const detailQuery = useQuery({
    queryKey: queryKeys.reportDetail(selectedReport?.id ?? ""),
    queryFn: () => api.getReportDetail(selectedReport?.id ?? ""),
    enabled: Boolean(selectedReport?.id),
  });

  return {
    chartMetric,
    detail: detailQuery.data,
    isDetailLoading: detailQuery.isLoading,
    reports,
    selectedReport,
    setChartMetric,
    setSelectedId,
  };
}
