import type { Dispatch, SetStateAction } from "react";
import { SelectField } from "../../../components/common/SelectField";
import { Input } from "../../../components/common/Input";
import { stepStrategyOptions } from "../constants";
import type { WorkbenchForm } from "../types";

type StaircaseConfigSectionProps = {
  form: WorkbenchForm;
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
};

export function StaircaseConfigSection({
  form,
  setForm,
}: StaircaseConfigSectionProps) {
  return (
    <>
      <div className="form-grid">
        <Input
          label="起始并发"
          min={1}
          type="number"
          value={form.start_concurrency}
          onChange={(event) =>
            setForm({
              ...form,
              start_concurrency: Number(event.target.value),
            })
          }
        />
        <Input
          label="结束并发"
          min={1}
          type="number"
          value={form.end_concurrency}
          onChange={(event) =>
            setForm({
              ...form,
              end_concurrency: Number(event.target.value),
            })
          }
        />
      </div>
      <SelectField
        label="阶梯方式"
        onChange={(step_strategy) => setForm({ ...form, step_strategy })}
        options={stepStrategyOptions}
        value={form.step_strategy}
      />
      <div className="form-grid">
        <Input
          label={form.step_strategy === "linear" ? "步长" : "倍率"}
          min={1}
          type="number"
          value={form.step_value}
          onChange={(event) =>
            setForm({ ...form, step_value: Number(event.target.value) })
          }
        />
        <Input
          label="每阶段请求轮次"
          hint="每轮会按当前并发发起一批请求，并等待本轮请求完成。"
          min={1}
          type="number"
          value={form.stage_sample_rounds}
          onChange={(event) =>
            setForm({
              ...form,
              stage_sample_rounds: Number(event.target.value),
              stage_duration_seconds: Number(event.target.value),
            })
          }
        />
      </div>
    </>
  );
}
