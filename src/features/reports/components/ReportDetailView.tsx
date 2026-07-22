import { useEffect, useRef, useState } from "react";
import type { ReportDetail } from "../../../types/api";
import {
  getReportChartTabs,
  getReportKpis,
  getStageColumns,
} from "../domain/reportDefinitions";
import type { ChartMetric } from "../types";
import { ReportErrorSection } from "./ReportErrorSection";
import { ReportEvidenceSection } from "./ReportEvidenceSection";
import { ReportHeroSection } from "./ReportHeroSection";
import { ReportKpiGrid } from "./ReportKpiGrid";
import { ReportRecommendationSection } from "./ReportRecommendationSection";
import { ReportRequestLogsSection } from "./ReportRequestLogsSection";
import { ReportSpecialtySection } from "./ReportSpecialtySection";
import { ReportStageTableSection } from "./ReportStageTableSection";
import { ReportTrendSection } from "./ReportTrendSection";

type ReportDetailViewProps = {
  detail: ReportDetail;
  chartMetric: ChartMetric;
  onChartMetricChange: (metric: ChartMetric) => void;
};

export function ReportDetailView({
  detail,
  chartMetric,
  onChartMetricChange,
}: ReportDetailViewProps) {
  const requestLogsRef = useRef<HTMLDivElement>(null);
  const [requestStageFilter, setRequestStageFilter] = useState<number | undefined>();
  const kpis = getReportKpis(detail);
  const tabs = getReportChartTabs(detail.model_type);
  const effectiveChartMetric = tabs.some((tab) => tab.key === chartMetric)
    ? chartMetric
    : tabs[0].key;
  const stageColumns = getStageColumns(detail.model_type);

  useEffect(() => {
    setRequestStageFilter(undefined);
  }, [detail.summary.task_id]);

  const viewStageRequests = (stageIndex: number) => {
    setRequestStageFilter(stageIndex);
    requestAnimationFrame(() => {
      requestLogsRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
    });
  };

  return (
    <div className="report-detail">
      <ReportHeroSection detail={detail} />
      <ReportKpiGrid kpis={kpis} />
      <ReportEvidenceSection detail={detail} />
      <ReportSpecialtySection detail={detail} />
      <ReportTrendSection
        chartMetric={effectiveChartMetric}
        detail={detail}
        onChartMetricChange={onChartMetricChange}
        tabs={tabs}
      />
      <ReportStageTableSection
        detail={detail}
        stageColumns={stageColumns}
        onViewRequests={
          detail.request_log_meta.total_records > 0 ? viewStageRequests : undefined
        }
      />
      <ReportErrorSection detail={detail} />
      <div className="report-request-logs-anchor" ref={requestLogsRef}>
        <ReportRequestLogsSection
          detail={detail}
          stageFilter={requestStageFilter}
          onStageFilterChange={setRequestStageFilter}
        />
      </div>
      <ReportRecommendationSection detail={detail} />
    </div>
  );
}
