import { useState } from "react";
import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../../api/client";
import { queryKeys } from "../../api/queryKeys";
import { Button } from "../../components/common/Button";
import { Input } from "../../components/common/Input";
import { Lock, Network, Settings2, SlidersHorizontal } from "../../components/common/icons";
import { PageHeader } from "../../components/common/PageHeader";
import { useToast } from "../../components/common/Toast";
import { Toggle } from "../../components/common/Toggle";
import { SelectField } from "../../components/common/SelectField";
import { SettingsPanel } from "../../features/settings/components/SettingsPanel";
import type { AppConfig, BenchmarkEngineMode, DataMode } from "../../types/api";

const defaultConfig: AppConfig = {
  data_mode: "mock",
  benchmark_engine: "mock",
};

export function Settings() {
  const queryClient = useQueryClient();
  const { pushToast } = useToast();
  const [configDraft, setConfigDraft] = useState<AppConfig>(defaultConfig);
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
      await queryClient.invalidateQueries({ queryKey: queryKeys.appConfig() });
      pushToast({
        title: "系统设置已保存",
        description: result.restart_required
          ? "配置已写入 Tauri config.json，数据源切换会在应用重启后生效。"
          : "配置已写入 Tauri config.json。",
        tone: "success" as const,
      });
    },
    onError: (error) => {
      pushToast({
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
    pushToast({
      title,
      description: "该配置项当前是界面占位，后续会接入对应的 Tauri 设置服务。",
      tone: "success" as const,
    });
  };

  return (
    <div className="page">
      <PageHeader
        eyebrow="系统配置"
        title="系统设置"
        description="配置默认 SLA、安全策略、网络访问和报告模板，让压测任务拥有一致的执行边界。"
        actions={
          <Button
            variant="primary"
            loading={saveConfigMutation.isPending}
            disabled={configQuery.isLoading}
            onClick={saveAppConfig}
          >
            保存设置
          </Button>
        }
      />

      <div className="settings-grid">
        <SettingsPanel
          icon={<Settings2 size={20} />}
          title="基础设置"
          description="系统名称、语言、时区和默认工作目录。"
        >
          <Input label="系统名称" defaultValue="LLMBench" />
          <SelectField<DataMode>
            label="数据来源"
            value={configDraft.data_mode}
            disabled={configQuery.isLoading}
            onChange={(value) => updateConfigDraft("data_mode", value)}
            options={[
              { value: "mock", label: "Rust Mock", description: "使用后端内存数据，不持久化历史记录" },
              { value: "sqlite", label: "SQLite", description: "使用本地数据库持久化服务商、任务、过程指标和报告" },
            ]}
          />
          <SelectField<BenchmarkEngineMode>
            label="压测引擎"
            value={configDraft.benchmark_engine}
            disabled={configQuery.isLoading}
            onChange={(value) => updateConfigDraft("benchmark_engine", value)}
            options={[
              { value: "mock", label: "Mock 引擎", description: "生成模拟指标，用于演示和离线验证页面流程" },
              {
                value: "openai_compatible",
                label: "OpenAI Compatible",
                description: "调用真实 Chat 接口压测，切换后需重启并重新创建任务",
              },
            ]}
          />
          <p className="settings-note">
            SQLite 只决定数据是否落到本地数据库；报告是否为真实接口实测，取决于创建任务时选择的压测引擎。
          </p>
          <SelectField
            label="默认语言"
            value="zh-CN"
            onChange={() => undefined}
            options={[
              { value: "zh-CN", label: "简体中文", description: "面向本地交付团队" },
              { value: "en-US", label: "English", description: "英文报告预留" },
            ]}
          />
        </SettingsPanel>
        <SettingsPanel
          icon={<SlidersHorizontal size={20} />}
          title="默认 SLA"
          description="P95、P99、错误率、TTFT 和自动停止阈值。"
        >
          <Input label="P95 阈值" defaultValue="5000" suffix="ms" />
          <Input label="最低成功率" defaultValue="99" suffix="%" />
          <Input label="自动停止连续失败阶段" defaultValue="2" />
        </SettingsPanel>
        <SettingsPanel
          icon={<Lock size={20} />}
          title="安全设置"
          description="API Key 安全存储、脱敏、请求响应落盘策略。"
        >
          <Toggle
            checked={savePayload}
            label="保存完整请求 / 响应正文"
            onChange={setSavePayload}
          />
          <SelectField
            label="API Key 展示策略"
            value="masked"
            onChange={() => undefined}
            options={[
              { value: "masked", label: "始终脱敏", description: "前端、日志和报告不展示明文" },
              { value: "confirm", label: "确认后短暂显示", description: "需要二次确认" },
            ]}
          />
          <p className="settings-note">
            试点版本会将 API Key 明文保存在本地数据源中，仅用于真实压测调用；正式交付需替换为系统密钥库。
          </p>
        </SettingsPanel>
        <SettingsPanel
          icon={<Network size={20} />}
          title="网络设置"
          description="代理、自签名证书、TLS 校验和 timeout。"
        >
          <Toggle checked={tlsVerify} label="校验 TLS 证书" onChange={setTlsVerify} />
          <Input label="HTTP 代理" placeholder="http://127.0.0.1:7890" />
          <Input label="请求 Timeout" defaultValue="60" suffix="s" />
        </SettingsPanel>
        <SettingsPanel
          icon={<Settings2 size={20} />}
          title="报告模板"
          description="配置售前版、运维版和详细版报告的默认模板。"
        >
          <SelectField
            label="默认模板"
            value={template}
            onChange={setTemplate}
            options={[
              { value: "交付摘要版", label: "交付摘要版", description: "结论优先" },
              { value: "运维容量版", label: "运维容量版", description: "指标优先" },
              { value: "详细审计版", label: "详细审计版", description: "证据优先" },
            ]}
          />
          <Button onClick={() => pushSaved("报告模板已更新")}>应用模板</Button>
        </SettingsPanel>
      </div>
    </div>
  );
}

