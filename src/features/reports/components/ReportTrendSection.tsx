import { RealtimeChart } from "../../../charts/RealtimeChart";
import { Card } from "../../../components/ui/Card";
import { Tabs } from "../../../components/ui/Tabs";
import type { ReportDetail } from "../../../types/api";
import type { ChartMetric } from "../types";
import type { TabItem } from "../../../components/ui/Tabs";

type ReportTrendSectionProps = {
  chartMetric: ChartMetric;
  detail: ReportDetail;
  onChartMetricChange: (metric: ChartMetric) => void;
  tabs: Array<TabItem<ChartMetric>>;
};

export function ReportTrendSection({
  chartMetric,
  detail,
  onChartMetricChange,
  tabs,
}: ReportTrendSectionProps) {
  return (
    <Card
      title="性能趋势"
      action={
        <Tabs
          items={tabs}
          onChange={onChartMetricChange}
          value={chartMetric}
        />
      }
    >
      <RealtimeChart data={detail.trends} metric={chartMetric} />
    </Card>
  );
}
