export const datasetTypes = [
  { key: "全部", label: "全部" },
  { key: "Chat", label: "文本生成" },
  { key: "Embedding", label: "向量嵌入" },
  { key: "Vision", label: "视觉多模态" },
  { key: "Reranker", label: "重排序" },
] as const;

export type DatasetTypeFilter = (typeof datasetTypes)[number]["key"];
