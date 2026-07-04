import { Card } from "../../../components/common/Card";
import { Tabs } from "../../../components/common/Tabs";
import { datasetTypes, type DatasetTypeFilter } from "../constants";

type DatasetTypeTabsProps = {
  value: DatasetTypeFilter;
  onChange: (value: DatasetTypeFilter) => void;
};

export function DatasetTypeTabs({ value, onChange }: DatasetTypeTabsProps) {
  return (
    <Card title="数据类型" eyebrow="类型筛选">
      <Tabs
        className="type-tabs"
        items={datasetTypes}
        onChange={onChange}
        value={value}
      />
    </Card>
  );
}
