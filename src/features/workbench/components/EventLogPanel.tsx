import { Card } from "../../../components/common/Card";

type EventLogPanelProps = {
  logs: string[];
};

export function EventLogPanel({ logs }: EventLogPanelProps) {
  return (
    <Card title="事件日志" eyebrow="Events">
      <div className="event-log">
        {logs.length ? (
          logs.map((log, index) => <div key={`${log}-${index}`}>{log}</div>)
        ) : (
          <div className="empty-line">等待任务启动。</div>
        )}
      </div>
    </Card>
  );
}
