import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../../api/client";
import { queryKeys } from "../../api/queryKeys";
import { Button } from "../../components/ui/Button";
import { Input } from "../../components/ui/Input";
import { FileText, Gauge, Lock, Network, Settings2 } from "../../components/ui/icons";
import { useNotification } from "../../components/ui/Notification";
import { Toggle } from "../../components/ui/Toggle";
import { SelectField } from "../../components/ui/SelectField";
import {
  SettingRow,
  SettingsPanel,
} from "../../features/settings/components/SettingsPanel";
import type {
  AppConfig,
  BenchmarkEngineMode,
  DataMode,
  NotificationPosition,
} from "../../types/api";

const defaultConfig: AppConfig = {
  data_mode: "mock",
  benchmark_engine: "mock",
  notification_position: "top-right",
};

const settingsSections = [
  {
    key: "general",
    title: "常规",
    description: "数据来源、压测引擎和语言",
    icon: <Settings2 size={18} />,
  },
  {
    key: "defaults",
    title: "压测默认值",
    description: "SLA、成功率和保护阈值",
    icon: <Gauge size={18} />,
  },
  {
    key: "network",
    title: "网络",
    description: "代理、TLS 和请求 timeout",
    icon: <Network size={18} />,
  },
  {
    key: "security",
    title: "安全",
    description: "正文采集和 API Key 展示",
    icon: <Lock size={18} />,
  },
  {
    key: "report",
    title: "报告",
    description: "默认交付模板",
    icon: <FileText size={18} />,
  },
] as const;

type SettingsSectionKey = (typeof settingsSections)[number]["key"];

export function Settings() {
  const queryClient = useQueryClient();
  const { notify, setPosition } = useNotification();
  const [configDraft, setConfigDraft] = useState<AppConfig>(defaultConfig);
  const [activeSection, setActiveSection] =
    useState<SettingsSectionKey>("general");
  const [savePayload, setSavePayload] = useState(false);
  const [tlsVerify, setTlsVerify] = useState(true);
  const [template, setTemplate] = useState("交付摘要版");

  const configQuery = useQuery({
    queryKey: queryKeys.appConfig(),
    queryFn: api.getAppConfig,
  });

  const saveConfigMutation = useMutation({
    mutationFn: api.updateAppConfig,
    onSuccess: async (result) => {
      setConfigDraft(result.config);
      setPosition(result.config.notification_position);
      await queryClient.invalidateQueries();
      notify({
        title: "系统设置已保存",
        description: result.restart_required
          ? "配置已写入 Tauri config.json，部分系统级配置会在应用重启后生效。"
          : "配置已立即生效，并已写入 Tauri config.json。",
        tone: "success" as const,
      });
    },
    onError: (error) => {
      notify({
        title: "系统设置保存失败",
        description: error instanceof Error ? error.message : String(error),
        tone: "danger" as const,
      });
    },
  });

  useEffect(() => {
    if (configQuery.data) {
      setConfigDraft(configQuery.data);
    }
  }, [configQuery.data]);

  const updateConfigDraft = <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
    setConfigDraft((current) => ({ ...current, [key]: value }));
  };

  const saveAppConfig = () => {
    saveConfigMutation.mutate(configDraft);
  };

  const pushSaved = (title: string) => {
    notify({
      title,
      description: "该配置项当前是界面占位，后续会接入对应的 Tauri 设置服务。",
      tone: "success" as const,
    });
  };

  const activeSectionDetail =
    settingsSections.find((section) => section.key === activeSection) ??
    settingsSections[0];

  return (
    <div className="page settings-page">
      <div className="settings-toolbar">
        <div className="settings-toolbar-copy">
          <h1>系统设置</h1>
          <span>默认 SLA、安全、网络和报告模板</span>
        </div>
        <div className="settings-toolbar-meta" aria-label="当前配置状态">
          <span className="settings-source-chip">本地配置</span>
          <StatusItem label="数据源" value={dataModeLabel(configDraft.data_mode)} />
          <StatusItem
            label="压测引擎"
            value={engineModeLabel(configDraft.benchmark_engine)}
          />
          <Button
            variant="primary"
            loading={saveConfigMutation.isPending}
            disabled={configQuery.isLoading}
            onClick={saveAppConfig}
          >
            保存设置
          </Button>
        </div>
      </div>

      <div className="settings-workspace">
        <nav className="settings-nav" aria-label="系统设置分类">
          {settingsSections.map((section) => (
            <button
              key={section.key}
              type="button"
              className={`settings-nav-item ${
                activeSection === section.key ? "active" : ""
              }`}
              onClick={() => setActiveSection(section.key)}
            >
              <span className="settings-nav-icon">{section.icon}</span>
              <span className="settings-nav-copy">
                <strong>{section.title}</strong>
                <em>{section.description}</em>
              </span>
            </button>
          ))}
        </nav>

        <SettingsPanel
          icon={activeSectionDetail.icon}
          title={activeSectionDetail.title}
          description={activeSectionDetail.description}
        >
          {activeSection === "general" && (
            <>
              <SettingRow
                label="系统名称"
                description="显示在窗口、报告和本地交付材料中的产品名称。"
              >
                <Input defaultValue="MyLLMBenchmark" />
              </SettingRow>
              <SettingRow
                label="数据来源"
                description="决定服务商、任务、指标和报告是否落到本地数据库。"
              >
                <SelectField<DataMode>
                  value={configDraft.data_mode}
                  disabled={configQuery.isLoading}
                  onChange={(value) => updateConfigDraft("data_mode", value)}
                  options={[
                    {
                      value: "mock",
                      label: "Rust Mock",
                      description: "使用后端内存数据，不持久化历史记录",
                    },
                    {
                      value: "sqlite",
                      label: "SQLite",
                      description:
                        "使用本地数据库持久化服务商、任务、过程指标和报告",
                    },
                  ]}
                />
              </SettingRow>
              <SettingRow
                label="压测引擎"
                description="决定新任务使用模拟指标还是真实 OpenAI Compatible / Jina 请求。"
              >
                <SelectField<BenchmarkEngineMode>
                  value={configDraft.benchmark_engine}
                  disabled={configQuery.isLoading}
                  onChange={(value) =>
                    updateConfigDraft("benchmark_engine", value)
                  }
                  options={[
                    {
                      value: "mock",
                      label: "Mock 引擎",
                      description: "生成模拟指标，用于演示和离线验证页面流程",
                    },
                    {
                      value: "openai_compatible",
                      label: "OpenAI Compatible",
                      description:
                        "调用真实 OpenAI Compatible / Jina 接口压测，保存后立即用于新任务",
                    },
                  ]}
                />
              </SettingRow>
              <SettingRow
                label="默认语言"
                description="控制默认界面和后续报告模板语言。"
              >
                <SelectField
                  value="zh-CN"
                  onChange={() => undefined}
                  options={[
                    {
                      value: "zh-CN",
                      label: "简体中文",
                      description: "面向本地交付团队",
                    },
                    {
                      value: "en-US",
                      label: "English",
                      description: "英文报告预留",
                    },
                  ]}
                />
              </SettingRow>
              <SettingRow
                label="应用通知位置"
                description="控制带标题和详情的应用通知显示位置；单行 Toast 固定显示在顶部中央。"
              >
                <SelectField<NotificationPosition>
                  value={configDraft.notification_position}
                  disabled={configQuery.isLoading}
                  onChange={(value) =>
                    updateConfigDraft("notification_position", value)
                  }
                  options={[
                    {
                      value: "top-right",
                      label: "右上角（推荐）",
                      description: "避开主工作区和底部状态栏",
                    },
                    {
                      value: "top-left",
                      label: "左上角",
                      description: "靠近应用导航区域",
                    },
                    {
                      value: "bottom-right",
                      label: "右下角",
                      description: "显示在底部状态栏上方",
                    },
                    {
                      value: "bottom-left",
                      label: "左下角",
                      description: "显示在导航栏右侧的底部区域",
                    },
                  ]}
                />
              </SettingRow>
              <p className="settings-note">
                SQLite 只决定数据是否落到本地数据库；报告是否为真实接口实测，取决于创建任务时选择的压测引擎。
              </p>
            </>
          )}

          {activeSection === "defaults" && (
            <>
              <SettingRow
                label="P95 阈值"
                description="报告和运行保护使用的默认 P95 延迟目标。"
              >
                <Input defaultValue="5000" suffix="ms" />
              </SettingRow>
              <SettingRow
                label="最低成功率"
                description="低于该成功率时，报告会标记 SLA 风险。"
              >
                <Input defaultValue="99" suffix="%" />
              </SettingRow>
              <SettingRow
                label="自动停止连续失败阶段"
                description="保护性停止策略启用时使用的连续失败阶段阈值。"
              >
                <Input defaultValue="2" />
              </SettingRow>
            </>
          )}

          {activeSection === "network" && (
            <>
              <SettingRow
                label="校验 TLS 证书"
                description="关闭后可连接自签名证书服务，仅建议在内网联调时使用。"
              >
                <div className="settings-toggle-control">
                  <Toggle
                    checked={tlsVerify}
                    ariaLabel="校验 TLS 证书"
                    onChange={setTlsVerify}
                  />
                  <span>{tlsVerify ? "已开启" : "已关闭"}</span>
                </div>
              </SettingRow>
              <SettingRow
                label="HTTP 代理"
                description="用于内网网关、抓包代理或无法直连模型服务时的网络出口。"
              >
                <Input placeholder="http://127.0.0.1:7890" />
              </SettingRow>
              <SettingRow
                label="请求 Timeout"
                description="单次服务商请求的默认超时时间。"
              >
                <Input defaultValue="60" suffix="s" />
              </SettingRow>
            </>
          )}

          {activeSection === "security" && (
            <>
              <SettingRow
                label="保存完整请求 / 响应正文"
                description="开启后本地会保存 Prompt 和模型响应正文，便于审计单次请求。"
              >
                <div className="settings-toggle-control">
                  <Toggle
                    checked={savePayload}
                    ariaLabel="保存完整请求 / 响应正文"
                    onChange={setSavePayload}
                  />
                  <span>{savePayload ? "已开启" : "已关闭"}</span>
                </div>
              </SettingRow>
              <SettingRow
                label="API Key 展示策略"
                description="控制界面、日志和报告中的密钥展示方式。"
              >
                <SelectField
                  value="masked"
                  onChange={() => undefined}
                  options={[
                    {
                      value: "masked",
                      label: "始终脱敏",
                      description: "前端、日志和报告不展示明文",
                    },
                    {
                      value: "confirm",
                      label: "确认后短暂显示",
                      description: "需要二次确认",
                    },
                  ]}
                />
              </SettingRow>
              <p className="settings-note settings-note-warning">
                试点版本会将 API Key 明文保存在本地数据源中，仅用于真实压测调用；正式交付需替换为系统密钥库。
              </p>
            </>
          )}

          {activeSection === "report" && (
            <>
              <SettingRow
                label="默认模板"
                description="生成报告时默认选中的交付模板。"
              >
                <div className="settings-control-with-action">
                  <SelectField
                    value={template}
                    onChange={setTemplate}
                    options={[
                      {
                        value: "交付摘要版",
                        label: "交付摘要版",
                        description: "结论优先",
                      },
                      {
                        value: "运维容量版",
                        label: "运维容量版",
                        description: "指标优先",
                      },
                      {
                        value: "详细审计版",
                        label: "详细审计版",
                        description: "证据优先",
                      },
                    ]}
                  />
                  <Button onClick={() => pushSaved("报告模板已更新")}>
                    应用模板
                  </Button>
                </div>
              </SettingRow>
              <p className="settings-note">
                报告模板设置当前为界面占位，导出弹窗仍可在生成报告时选择具体模板。
              </p>
            </>
          )}
        </SettingsPanel>
      </div>
    </div>
  );
}

function StatusItem({ label, value }: { label: string; value: string }) {
  return (
    <span className="settings-status-item">
      <small>{label}</small>
      <strong>{value}</strong>
    </span>
  );
}

function dataModeLabel(value: DataMode) {
  return value === "sqlite" ? "SQLite" : "Rust Mock";
}

function engineModeLabel(value: BenchmarkEngineMode) {
  return value === "openai_compatible" ? "OpenAI Compatible" : "Mock";
}

