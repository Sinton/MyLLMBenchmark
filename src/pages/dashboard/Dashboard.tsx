import { Link } from "react-router-dom";
import { Badge, statusLabel, statusTone } from "../../components/common/Badge";
import { Card } from "../../components/common/Card";
import {
  ArrowRight,
  Building2,
  Database,
  FileText,
} from "../../components/common/icons";
import { InlineAlert } from "../../components/common/InlineAlert";
import { MetricCard } from "../../components/common/MetricCard";
import { useDashboardSummary } from "../../features/dashboard/hooks/useDashboardSummary";

export function Dashboard() {
  const { data, isLoading } = useDashboardSummary();
  const latestTask = data?.recent_tasks[0];
  const latestTaskLink = latestTask ? taskWorkbenchLink(latestTask.id) : "/workbench";

  return (
    <div className="page">
      <div className="metric-grid">
        <MetricCard label="模型服务商" value={isLoading ? "-" : data?.providers ?? 0} />
        <MetricCard label="模型数量" value={isLoading ? "-" : data?.models ?? 0} />
        <MetricCard label="压测任务" value={isLoading ? "-" : data?.tasks ?? 0} />
        <MetricCard label="测试报告" value={isLoading ? "-" : data?.reports ?? 0} />
      </div>

      <div className="two-column">
        <Card
          title="最近压测"
          eyebrow="任务"
          action={
            <Link className="text-link" to={latestTaskLink}>
              {latestTask ? "查看最新任务" : "查看工作台"} <ArrowRight size={14} />
            </Link>
          }
        >
          <div className="list-stack">
            {data?.recent_tasks.length ? (
              data.recent_tasks.map((task) => (
                <Link className="list-row list-row-link" key={task.id} to={taskWorkbenchLink(task.id)}>
                  <div>
                    <strong>{task.model_name}</strong>
                    <span>{task.dataset_name}</span>
                  </div>
                  <Badge tone={statusTone(task.status)}>{statusLabel(task.status)}</Badge>
                </Link>
              ))
            ) : (
              <InlineAlert title="还没有压测任务">
                进入工作台启动一次压测。
              </InlineAlert>
            )}
          </div>
        </Card>

        <Card
          title="最近报告"
          eyebrow="报告"
          action={
            <Link className="text-link" to="/reports">
              查看报告 <ArrowRight size={14} />
            </Link>
          }
        >
          <div className="list-stack">
            {data?.recent_reports.length ? (
              data.recent_reports.map((report) => (
                <div className="list-row" key={report.id}>
                  <div>
                    <strong>{report.model_name}</strong>
                    <span>推荐并发 {report.recommended_concurrency}</span>
                  </div>
                  <Badge tone="success">已生成</Badge>
                </div>
              ))
            ) : (
              <InlineAlert title="还没有历史报告">
                报告会在压测结束后生成。
              </InlineAlert>
            )}
          </div>
        </Card>
      </div>

      <div className="quick-grid">
        <Link to="/providers" className="quick-action">
          <Building2 size={20} />
          <div>
            <strong>配置模型服务商</strong>
            <span>添加 Base URL 和模型入口</span>
          </div>
        </Link>
        <Link to="/datasets" className="quick-action">
          <Database size={20} />
          <div>
            <strong>查看测试数据集</strong>
            <span>确认 token 分布和样本类型</span>
          </div>
        </Link>
        <Link to="/reports" className="quick-action">
          <FileText size={20} />
          <div>
            <strong>交付容量报告</strong>
            <span>查看推荐并发和容量建议</span>
          </div>
        </Link>
      </div>
    </div>
  );
}

function taskWorkbenchLink(taskId: string) {
  return `/workbench?taskId=${encodeURIComponent(taskId)}`;
}
