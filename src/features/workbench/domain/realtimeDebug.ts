type RealtimeDebugChannel = "event" | "polling";

const PREFIX = "[LLMBench realtime]";

export function debugRealtime(
  channel: RealtimeDebugChannel,
  message: string,
  details?: Record<string, unknown>,
) {
  if (!isRealtimeDebugEnabled()) return;

  if (details) {
    console.debug(`${PREFIX} [${channel}] ${message}`, details);
    return;
  }

  console.debug(`${PREFIX} [${channel}] ${message}`);
}

export function warnRealtime(
  channel: RealtimeDebugChannel,
  message: string,
  details?: Record<string, unknown>,
) {
  if (details) {
    console.warn(`${PREFIX} [${channel}] ${message}`, details);
    return;
  }

  console.warn(`${PREFIX} [${channel}] ${message}`);
}

function isRealtimeDebugEnabled() {
  return globalThis.localStorage?.getItem("LLMBENCH_REALTIME_DEBUG") !== "0";
}
