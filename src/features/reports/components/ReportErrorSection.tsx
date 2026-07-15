import { Card } from "../../../components/ui/Card";
import { ProgressBar } from "../../../components/ui/ProgressBar";
import type { ReportDetail } from "../../../types/api";

type ReportErrorSectionProps = {
  detail: ReportDetail;
};

export function ReportErrorSection({ detail }: ReportErrorSectionProps) {
  return (
    <Card title="错误分布">
      {detail.errors.length ? (
        <div className="error-buckets">
          {detail.errors.map((bucket) => (
            <div className="error-bucket" key={bucket.label}>
              <div>
                <span>{bucket.label}</span>
                <strong>{bucket.value}</strong>
              </div>
              <ProgressBar label={`${bucket.label} ${bucket.percent}%`} value={bucket.percent} />
            </div>
          ))}
        </div>
      ) : (
        <div className="report-empty-note">
          <strong>未记录错误</strong>
          <span>本次压测过程没有写入 timeout、HTTP 错误或连接异常。</span>
        </div>
      )}
    </Card>
  );
}
