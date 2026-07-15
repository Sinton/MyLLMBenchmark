import { Button } from "../../../components/ui/Button";
import { Play } from "../../../components/ui/icons";

type StartActionFooterProps = {
  canSubmitStart: boolean;
  startBlockReason: string | null;
  startPending: boolean;
};

export function StartActionFooter({
  canSubmitStart,
  startBlockReason,
  startPending,
}: StartActionFooterProps) {
  return (
    <Button
      disabled={!canSubmitStart}
      icon={<Play size={16} />}
      loading={startPending}
      type="submit"
      variant="primary"
    >
      {startPending ? "启动中" : startBlockReason ? "检查配置" : "开始压测"}
    </Button>
  );
}
