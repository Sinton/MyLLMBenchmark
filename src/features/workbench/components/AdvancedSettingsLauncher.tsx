import { ArrowRight, SlidersHorizontal } from "../../../components/ui/icons";

type AdvancedSettingsLauncherProps = {
  summary?: string;
  onOpen: () => void;
};

export function AdvancedSettingsLauncher({
  summary = "负载 / 保护 / 证据采集",
  onOpen,
}: AdvancedSettingsLauncherProps) {
  return (
    <div className="advanced-settings-launcher">
      <button
        className="advanced-settings-entry"
        onClick={onOpen}
        type="button"
      >
        <span className="advanced-settings-entry-icon">
          <SlidersHorizontal size={16} />
        </span>
        <span className="advanced-settings-entry-copy">
          <strong>高级设置</strong>
          <span>{summary}</span>
        </span>
        <ArrowRight
          aria-hidden="true"
          className="advanced-settings-entry-action"
          size={16}
        />
      </button>
    </div>
  );
}
