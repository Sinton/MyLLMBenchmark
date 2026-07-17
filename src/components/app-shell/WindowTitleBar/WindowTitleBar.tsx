import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMemo } from "react";

type DesktopPlatform = "windows" | "macos" | "linux";

export function WindowTitleBar() {
  const platform = useMemo(detectDesktopPlatform, []);

  async function handleDragStart() {
    try {
      await getCurrentWindow().startDragging();
    } catch {
      // Browser preview has no Tauri window; native drag is only available in app.
    }
  }

  async function handleToggleMaximize() {
    try {
      await getCurrentWindow().toggleMaximize();
    } catch {
      // No-op outside Tauri.
    }
  }

  async function handleMinimize() {
    try {
      await getCurrentWindow().minimize();
    } catch {
      // No-op outside Tauri.
    }
  }

  async function handleClose() {
    try {
      await getCurrentWindow().close();
    } catch {
      // No-op outside Tauri.
    }
  }

  const controls = (
    <div className="window-controls" data-window-controls>
      {platform === "macos" ? (
        <>
          <button
            aria-label="关闭窗口"
            className="window-control macos-close"
            onClick={handleClose}
            type="button"
          />
          <button
            aria-label="最小化窗口"
            className="window-control macos-minimize"
            onClick={handleMinimize}
            type="button"
          />
          <button
            aria-label="最大化窗口"
            className="window-control macos-maximize"
            onClick={handleToggleMaximize}
            type="button"
          />
        </>
      ) : (
        <>
          <button
            aria-label="最小化窗口"
            className="window-control windows-minimize"
            onClick={handleMinimize}
            type="button"
          >
            <span />
          </button>
          <button
            aria-label="最大化窗口"
            className="window-control windows-maximize"
            onClick={handleToggleMaximize}
            type="button"
          >
            <span />
          </button>
          <button
            aria-label="关闭窗口"
            className="window-control windows-close"
            onClick={handleClose}
            type="button"
          >
            <span />
          </button>
        </>
      )}
    </div>
  );

  return (
    <header className={`window-titlebar platform-${platform}`}>
      {platform === "macos" && controls}
      <div
        className="window-drag-region"
        data-tauri-drag-region
        onDoubleClick={handleToggleMaximize}
        onMouseDown={(event) => {
          if (event.detail === 1) {
            void handleDragStart();
          }
        }}
        >
        <div className="window-titlebar-brand" data-tauri-drag-region>
          <img
            alt=""
            aria-hidden="true"
            data-tauri-drag-region
            draggable={false}
            src="/logo.png"
          />
          <strong data-tauri-drag-region>MyLLMBenchmark</strong>
        </div>
      </div>
      {platform !== "macos" && controls}
    </header>
  );
}

function detectDesktopPlatform(): DesktopPlatform {
  const platform = navigator.platform.toLowerCase();
  const userAgent = navigator.userAgent.toLowerCase();

  if (platform.includes("mac") || userAgent.includes("mac os")) {
    return "macos";
  }

  if (platform.includes("win") || userAgent.includes("windows")) {
    return "windows";
  }

  return "linux";
}
