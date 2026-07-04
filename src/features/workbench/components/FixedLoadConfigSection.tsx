import type { Dispatch, SetStateAction } from "react";
import { Input } from "../../../components/common/Input";
import type { WorkbenchForm } from "../types";

type FixedLoadConfigSectionProps = {
  form: WorkbenchForm;
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
};

export function FixedLoadConfigSection({ form, setForm }: FixedLoadConfigSectionProps) {
  return (
    <div className="form-grid">
      <Input
        label="并发"
        min={1}
        type="number"
        value={form.concurrency}
        onChange={(event) =>
          setForm({ ...form, concurrency: Number(event.target.value) })
        }
      />
      <Input
        label="请求轮次"
        hint="固定并发下，每轮按当前并发发起一批请求。"
        min={1}
        max={300}
        type="number"
        value={form.duration_seconds}
        onChange={(event) =>
          setForm({
            ...form,
            duration_seconds: Number(event.target.value),
          })
        }
      />
    </div>
  );
}
