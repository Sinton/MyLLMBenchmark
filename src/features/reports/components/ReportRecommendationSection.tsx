import { Card } from "../../../components/common/Card";
import { AlertTriangle, BarChart3, ListChecks, TrendingUp } from "../../../components/common/icons";
import type { ReportDetail } from "../../../types/api";
import { RecommendationItem } from "./RecommendationItem";

type ReportRecommendationSectionProps = {
  detail: ReportDetail;
};

const recommendationTitles = ["上线限流", "SLA 告警", "模型专项", "复测策略"];

export function ReportRecommendationSection({ detail }: ReportRecommendationSectionProps) {
  return (
    <Card title="容量建议">
      <div className="recommendation-grid">
        {detail.recommendations.map((item, index) => (
          <RecommendationItem
            icon={getRecommendationIcon(index)}
            key={item}
            text={item}
            title={recommendationTitles[index] ?? "建议"}
          />
        ))}
      </div>
      <div className="report-callout">{detail.summary.recommendation}</div>
    </Card>
  );
}

function getRecommendationIcon(index: number) {
  if (index === 0) return <ListChecks size={18} />;
  if (index === 1) return <BarChart3 size={18} />;
  if (index === 2) return <TrendingUp size={18} />;
  return <AlertTriangle size={18} />;
}
