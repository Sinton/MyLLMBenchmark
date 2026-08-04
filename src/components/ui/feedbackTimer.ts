import { useCallback, useEffect, useRef } from "react";

export type PausableTimer = {
  cancel: () => void;
  pause: () => void;
  resume: () => void;
};

export function createPausableTimer(
  durationMs: number,
  onElapsed: () => void,
): PausableTimer {
  let remainingMs = durationMs;
  let startedAt = Date.now();
  let timer: ReturnType<typeof setTimeout> | null = null;

  const cancel = () => {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
  };

  const resume = () => {
    if (timer !== null || remainingMs <= 0) return;
    startedAt = Date.now();
    timer = setTimeout(() => {
      timer = null;
      remainingMs = 0;
      onElapsed();
    }, remainingMs);
  };

  const pause = () => {
    if (timer === null) return;
    remainingMs = Math.max(0, remainingMs - (Date.now() - startedAt));
    cancel();
  };

  resume();
  return { cancel, pause, resume };
}

export function usePausableAutoDismiss(
  durationMs: number | null,
  onDismiss: () => void,
) {
  const callbackRef = useRef(onDismiss);
  const timerRef = useRef<PausableTimer | null>(null);

  useEffect(() => {
    callbackRef.current = onDismiss;
  }, [onDismiss]);

  useEffect(() => {
    timerRef.current?.cancel();
    timerRef.current =
      durationMs === null
        ? null
        : createPausableTimer(durationMs, () => callbackRef.current());

    return () => {
      timerRef.current?.cancel();
      timerRef.current = null;
    };
  }, [durationMs]);

  return {
    pause: useCallback(() => timerRef.current?.pause(), []),
    resume: useCallback(() => timerRef.current?.resume(), []),
  };
}
