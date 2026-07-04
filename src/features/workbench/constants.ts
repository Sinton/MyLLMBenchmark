export const benchmarkModeOptions = [
  { value: "阶梯加压", label: "阶梯加压", description: "逐步提升并发" },
  { value: "固定并发", label: "固定并发", description: "稳定负载点" },
  { value: "固定 QPS", label: "固定 QPS", description: "模拟业务流量" },
  { value: "长稳压测", label: "长稳压测", description: "验证稳定性" },
];

export const stepStrategyOptions = [
  { value: "double", label: "倍增", description: "1 -> 2 -> 4 -> 8" },
  { value: "linear", label: "固定步长", description: "按固定并发增量推进" },
];

export const slaStopPolicyOptions = [
  {
    value: "continue_full_staircase",
    label: "继续完整阶梯",
    description: "即使某阶段不达标，也继续收集后续容量证据",
  },
  {
    value: "stop_on_failure",
    label: "保护性停止",
    description: "首个阶段不达标后停止，避免继续压垮服务",
  },
];
