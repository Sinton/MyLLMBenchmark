import { useState } from "react";
import { Button } from "../../../components/ui/Button";
import { Copy } from "../../../components/ui/icons";
import type { DatasetSamplePreview } from "../../../types/api";

type DatasetSampleExpandedRowProps = {
  sample: DatasetSamplePreview;
};

export function DatasetSampleExpandedRow({ sample }: DatasetSampleExpandedRowProps) {
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied" | "failed">(
    "idle",
  );

  const copyPrompt = async () => {
    try {
      await navigator.clipboard.writeText(sample.prompt);
      setCopyStatus("copied");
    } catch {
      setCopyStatus("failed");
    }
  };

  return (
    <section className="dataset-sample-expanded">
      <header>
        <div>
          <strong>完整 Prompt</strong>
          <span>样本 #{sample.sample_index + 1}</span>
        </div>
        <Button
          aria-label={`复制第 ${sample.sample_index + 1} 条样本 Prompt`}
          icon={<Copy aria-hidden="true" size={14} />}
          variant="ghost"
          onClick={copyPrompt}
        >
          {copyStatus === "copied"
            ? "已复制"
            : copyStatus === "failed"
              ? "复制失败"
              : "复制"}
        </Button>
      </header>
      <pre>{sample.prompt}</pre>
    </section>
  );
}
