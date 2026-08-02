import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import type { QueryClient } from "@tanstack/react-query";
import { listenToEvent } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import type {
  EndpointProbeBatchDetail,
  EndpointProbeBatchSummary,
  EndpointProbeResponseDeltaEvent,
  EndpointProbeRunDetail,
  EndpointProbeRunFinishedEvent,
  EndpointProbeRunStartedEvent,
} from "../../../types/api";
import {
  appendEndpointProbeDeltas,
  type EndpointProbeStreamBuffer,
} from "../domain/endpointProbePresentation";

type UseEndpointProbeEventsInput = {
  activeBatchId: string | null;
  queryClient: QueryClient;
  setActiveBatch: Dispatch<SetStateAction<EndpointProbeBatchDetail | null>>;
  setRunDetails: Dispatch<SetStateAction<Record<string, EndpointProbeRunDetail>>>;
};

export function useEndpointProbeEvents({
  activeBatchId,
  queryClient,
  setActiveBatch,
  setRunDetails,
}: UseEndpointProbeEventsInput) {
  const [listenersReady, setListenersReady] = useState(false);
  const [streamBuffers, setStreamBuffers] = useState<
    Record<string, EndpointProbeStreamBuffer>
  >({});
  const pendingDeltas = useRef<EndpointProbeResponseDeltaEvent[]>([]);
  const animationFrame = useRef<number | null>(null);

  const flushDeltas = useCallback(() => {
    animationFrame.current = null;
    const queued = pendingDeltas.current.splice(0);
    if (queued.length) {
      setStreamBuffers((current) => appendEndpointProbeDeltas(current, queued));
    }
  }, []);

  useEffect(() => {
    let disposed = false;
    let cleanup: Array<() => void> = [];
    Promise.allSettled([
      listenToEvent<EndpointProbeBatchSummary>("endpoint_probe:batch_started", (batch) => {
        setActiveBatch((current) =>
          current?.id === batch.id ? { ...current, ...batch } : current,
        );
      }),
      listenToEvent<EndpointProbeRunStartedEvent>("endpoint_probe:run_started", (event) => {
        setActiveBatch((current) =>
          current?.id === event.batch_id
            ? {
                ...current,
                runs: current.runs.map((run) =>
                  run.id === event.run_id ? { ...run, status: "running" } : run,
                ),
              }
            : current,
        );
      }),
      listenToEvent<EndpointProbeResponseDeltaEvent>(
        "endpoint_probe:response_delta",
        (event) => {
          pendingDeltas.current.push(event);
          if (animationFrame.current === null) {
            animationFrame.current = window.requestAnimationFrame(flushDeltas);
          }
        },
      ),
      listenToEvent<EndpointProbeRunFinishedEvent>(
        "endpoint_probe:run_finished",
        (event) => {
          pendingDeltas.current = pendingDeltas.current.filter(
            (delta) => delta.run_id !== event.run.id,
          );
          setRunDetails((current) => ({ ...current, [event.run.id]: event.run }));
          const finalText = event.run.response_text ?? event.run.response_preview;
          if (finalText) {
            setStreamBuffers((current) => ({
              ...current,
              [event.run.id]: {
                batchId: event.batch_id,
                text: finalText,
                lastSequence: Number.MAX_SAFE_INTEGER,
                finished: true,
              },
            }));
          }
          setActiveBatch((current) =>
            current?.id === event.batch_id
              ? {
                  ...current,
                  runs: current.runs.map((run) =>
                    run.id === event.run.id ? event.run : run,
                  ),
                }
              : current,
          );
          void invalidateEndpointProbeQueries(queryClient);
        },
      ),
      listenToEvent<EndpointProbeBatchSummary>("endpoint_probe:batch_finished", (batch) => {
        setActiveBatch((current) =>
          current?.id === batch.id ? { ...current, ...batch } : current,
        );
        void invalidateEndpointProbeQueries(queryClient);
      }),
    ]).then((results) => {
      cleanup = results.flatMap((result) =>
        result.status === "fulfilled" ? [result.value] : [],
      );
      if (disposed) {
        cleanup.forEach((unlisten) => unlisten());
        return;
      }
      setListenersReady(results.every((result) => result.status === "fulfilled"));
    });

    return () => {
      disposed = true;
      cleanup.forEach((unlisten) => unlisten());
      if (animationFrame.current !== null) {
        window.cancelAnimationFrame(animationFrame.current);
      }
    };
  }, [flushDeltas, queryClient, setActiveBatch, setRunDetails]);

  const resetStreams = useCallback(() => {
    pendingDeltas.current = [];
    if (animationFrame.current !== null) {
      window.cancelAnimationFrame(animationFrame.current);
      animationFrame.current = null;
    }
    setStreamBuffers({});
  }, []);

  const streamText = useMemo(
    () =>
      Object.fromEntries(
        Object.entries(streamBuffers)
          .filter(([, buffer]) => buffer.batchId === activeBatchId)
          .map(([runId, buffer]) => [runId, buffer.text]),
      ),
    [activeBatchId, streamBuffers],
  );

  return { listenersReady, resetStreams, streamText };
}

async function invalidateEndpointProbeQueries(queryClient: QueryClient) {
  await Promise.all([
    queryClient.invalidateQueries({ queryKey: ["endpoint-probe-batches"] }),
    queryClient.invalidateQueries({ queryKey: ["endpoint-probe-batch-detail"] }),
    queryClient.invalidateQueries({ queryKey: queryKeys.providers() }),
  ]);
}
