import { useState } from "react";
import { api } from "../../api/client";
import { Link } from "react-router-dom";
import { Button } from "../../components/common/Button";
import { Card } from "../../components/common/Card";
import { Download, FileText } from "../../components/common/icons";
import { EmptyState } from "../../components/common/EmptyState";
import { LoadingBlock } from "../../components/common/LoadingBlock";
import { PageHeader } from "../../components/common/PageHeader";
import { useToast } from "../../components/common/Toast";
import { ReportDetailView } from "../../features/reports/components/ReportDetailView";
import { ReportExportDialog } from "../../features/reports/components/ReportExportDialog";
import { ReportRail } from "../../features/reports/components/ReportRail";
import { useReportsView } from "../../features/reports/hooks/useReportsView";

export function Reports() {
  const [exportOpen, setExportOpen] = useState(false);
  const [exporting, setExporting] = useState(false);
  const { pushToast } = useToast();
  const reportsView = useReportsView();

  return (
    <div className="page reports-page">
      <PageHeader
        eyebrow="交付报告"
        title="测试报告"
        description="面向大模型黑盒压测的交付报告，集中展示容量结论、LLM 专项指标、阶段数据和上线建议。"
        actions={
          <Button
            disabled={!reportsView.selectedReport}
            icon={<Download size={16} />}
            onClick={() => setExportOpen(true)}
            variant="primary"
          >
            导出报告
          </Button>
        }
      />

      {reportsView.reports.length === 0 ? (
        <Card>
          <EmptyState
            action={
              <Link to="/workbench">
                <Button variant="primary">开始压测</Button>
              </Link>
            }
            description="进入压测工作台完成一次压测后，即可生成容量评估报告。"
            icon={<FileText size={28} />}
            title="还没有测试报告"
          />
        </Card>
      ) : (
        <div className="reports-workspace">
          <ReportRail
            reports={reportsView.reports}
            selectedId={reportsView.selectedReport?.id}
            onSelect={reportsView.setSelectedId}
          />
          {reportsView.isDetailLoading || !reportsView.detail ? (
            <Card>
              <LoadingBlock text="正在生成报告详情..." />
            </Card>
          ) : (
            <ReportDetailView
              detail={reportsView.detail}
              chartMetric={reportsView.chartMetric}
              onChartMetricChange={reportsView.setChartMetric}
            />
          )}
        </div>
      )}

      <ReportExportDialog
        exporting={exporting}
        open={exportOpen}
        report={reportsView.selectedReport}
        onClose={() => setExportOpen(false)}
        onExport={async ({ format, template }) => {
          if (!reportsView.selectedReport) {
            throw new Error("请先选择一份报告");
          }
          setExporting(true);
          try {
            const result = await api.exportReport({
              report_id: reportsView.selectedReport.id,
              format,
              template,
            });
            pushToast({
              title: "报告导出完成",
              description: result.file_name,
              tone: "success",
            });
            return result;
          } finally {
            setExporting(false);
          }
        }}
      />
    </div>
  );
}
