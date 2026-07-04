import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import type { DatasetTypeFilter } from "../constants";

export function useDatasetsCatalog() {
  const [activeType, setActiveType] = useState<DatasetTypeFilter>("全部");
  const { data: datasets = [] } = useQuery({
    queryKey: queryKeys.datasets(),
    queryFn: api.listDatasets,
  });

  const filtered = useMemo(
    () =>
      activeType === "全部"
        ? datasets
        : datasets.filter((dataset) => dataset.dataset_type === activeType),
    [activeType, datasets],
  );

  return {
    activeType,
    datasets,
    filtered,
    setActiveType,
  };
}
