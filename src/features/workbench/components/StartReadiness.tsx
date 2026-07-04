import { AlertCircle, CheckCircle2 } from "../../../components/common/icons";
import type { StartNotice } from "../types";

type StartReadinessProps = {
  notice: StartNotice | null;
  reason: string | null;
};

export function StartReadiness({ notice, reason }: StartReadinessProps) {
  const display = notice ?? {
    tone: reason ? "info" : "success",
    title: reason ? "启动前检查" : "配置已就绪",
    message: reason ?? "点击开始后会立即创建任务、追加日志，并在 1 秒内刷新实时指标。",
  };

  return (
    <div className={`start-readiness ${display.tone}`}>
      {display.tone === "success" ? (
        <CheckCircle2 size={17} />
      ) : (
        <AlertCircle size={17} />
      )}
      <div>
        <strong>{display.title}</strong>
        <span>{display.message}</span>
      </div>
    </div>
  );
}
