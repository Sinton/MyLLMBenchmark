import { useEffect, useRef } from "react";
import type { ECharts } from "echarts/core";
import { chartMetricMeta, chartTheme } from "./chartTheme";
import type { ChartMetric } from "../domain/modelMetrics";
import type { MetricsTick } from "../types/api";

type RealtimeChartProps = {
  data: MetricsTick[];
  emptyState?: {
    description: string;
    title: string;
    tone?: "idle" | "loading" | "waiting";
  };
  metric: ChartMetric;
};

let echartsLoader: Promise<typeof import("echarts/core")> | null = null;

export function RealtimeChart({ data, emptyState, metric }: RealtimeChartProps) {
  const ref = useRef<HTMLDivElement | null>(null);
  const chartRef = useRef<ECharts | null>(null);
  const optionRef = useRef(buildChartOption(data, metric));
  const isLiveWaiting =
    emptyState?.tone === "loading" || emptyState?.tone === "waiting";

  optionRef.current = buildChartOption(data, metric);

  useEffect(() => {
    if (!ref.current) return;
    let disposed = false;

    void loadEcharts().then((echarts) => {
      if (!ref.current || disposed) return;
      chartRef.current = echarts.init(ref.current, undefined, { renderer: "canvas" });
      chartRef.current.setOption(optionRef.current);
    });

    const resize = () => chartRef.current?.resize();
    window.addEventListener("resize", resize);
    return () => {
      disposed = true;
      window.removeEventListener("resize", resize);
      chartRef.current?.dispose();
      chartRef.current = null;
    };
  }, []);

  useEffect(() => {
    const chart = chartRef.current;
    if (!chart) return;

    chart.setOption(buildChartOption(data, metric));
  }, [data, metric]);

  return (
    <div className="chart-frame">
      <div className="chart-canvas" ref={ref} />
      {!data.length && (
        <div className={`chart-empty chart-empty-${emptyState?.tone ?? "idle"}`}>
          {isLiveWaiting && (
            <div className="chart-empty-loader" aria-hidden="true">
              <span />
              <span />
              <span />
            </div>
          )}
          <strong>{emptyState?.title ?? "等待实时指标"}</strong>
          <span>
            {emptyState?.description ??
              "压测启动后，首批 QPS、Latency、TTFT 通常会在 1 秒内到达。"}
          </span>
        </div>
      )}
    </div>
  );
}

async function loadEcharts() {
  if (!echartsLoader) {
    echartsLoader = Promise.all([
      import("echarts/core"),
      import("echarts/charts"),
      import("echarts/components"),
      import("echarts/renderers"),
    ]).then(([echarts, charts, components, renderers]) => {
      echarts.use([
        charts.LineChart,
        components.GridComponent,
        components.MarkLineComponent,
        components.TooltipComponent,
        renderers.CanvasRenderer,
      ]);
      return echarts;
    });
  }
  return echartsLoader;
}

function buildChartOption(data: MetricsTick[], metric: RealtimeChartProps["metric"]) {
  const meta = chartMetricMeta[metric];
  return {
      animationDuration: 180,
      grid: { left: 42, right: 18, top: 24, bottom: 34 },
      tooltip: {
        trigger: "axis",
        formatter: (params: unknown) => {
          const [item] = params as Array<{ dataIndex: number; value: number }>;
          const point = data[item.dataIndex];
          if (!point) return "";
          return [
            `第 ${point.elapsed_seconds} 轮`,
            `${meta.name}: ${item.value}${meta.unit}`,
            `QPS: ${point.qps}`,
            `Success: ${point.success_rate}%`,
          ].join("<br/>");
        },
      },
      xAxis: {
        type: "category",
        boundaryGap: false,
        data: data.map((item) => `${item.elapsed_seconds}`),
        axisLine: { lineStyle: { color: chartTheme.axis } },
        axisLabel: { color: chartTheme.label },
      },
      yAxis: {
        type: "value",
        axisLabel: { color: chartTheme.label },
        splitLine: { lineStyle: { color: chartTheme.splitLine } },
      },
      series: [
        {
          name: meta.name,
          type: "line",
          smooth: true,
          showSymbol: data.length < 2,
          symbolSize: 7,
          lineStyle: { width: 3, color: meta.color },
          areaStyle: { color: `${meta.color}18` },
          data: data.map((item) => pickValue(item, metric)),
          markLine:
            metric === "latency"
              ? {
                  symbol: "none",
                  lineStyle: { color: chartTheme.sla, type: "dashed" },
                  data: [{ yAxis: 5000, name: "SLA" }],
                }
              : undefined,
        },
      ],
    };
}

function pickValue(item: MetricsTick, metric: RealtimeChartProps["metric"]) {
  switch (metric) {
    case "latency":
      return item.latency_ms;
    case "ttft":
      return item.ttft_ms;
    case "qps":
      return item.qps;
    case "tps":
      return item.tps;
    case "success":
      return item.success_rate;
    case "errors":
      return item.errors;
  }
}
