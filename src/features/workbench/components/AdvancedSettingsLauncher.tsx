import { ArrowRight, SlidersHorizontal } from "../../../components/common/icons";

type AdvancedSettingsLauncherProps = {
  onOpen: () => void;
};

export function AdvancedSettingsLauncher({
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
          <span>负载参数、运行保护与证据采集</span>
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
