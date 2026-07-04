import type { Dispatch, SetStateAction } from "react";
import { SelectField } from "../../../components/common/SelectField";
import { benchmarkModeOptions } from "../constants";
import type { WorkbenchForm } from "../types";

type BenchmarkModeSectionProps = {
  form: WorkbenchForm;
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
};

export function BenchmarkModeSection({ form, setForm }: BenchmarkModeSectionProps) {
  return (
    <SelectField
      label="压测模式"
      onChange={(mode) => setForm({ ...form, mode })}
      options={benchmarkModeOptions}
      value={form.mode}
    />
  );
}
