import { afterEach, describe, expect, it, vi } from "vitest";
import { createPausableTimer } from "./feedbackTimer";

describe("createPausableTimer", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("pauses and resumes with the remaining duration", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-08-05T00:00:00Z"));
    const onElapsed = vi.fn();
    const timer = createPausableTimer(3_000, onElapsed);

    vi.advanceTimersByTime(1_000);
    timer.pause();
    vi.advanceTimersByTime(10_000);
    expect(onElapsed).not.toHaveBeenCalled();

    timer.resume();
    vi.advanceTimersByTime(1_999);
    expect(onElapsed).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(onElapsed).toHaveBeenCalledTimes(1);
  });

  it("does not elapse after cancellation", () => {
    vi.useFakeTimers();
    const onElapsed = vi.fn();
    const timer = createPausableTimer(3_000, onElapsed);

    timer.cancel();
    vi.advanceTimersByTime(3_000);

    expect(onElapsed).not.toHaveBeenCalled();
  });
});
