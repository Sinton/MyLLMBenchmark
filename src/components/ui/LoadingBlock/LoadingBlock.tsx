type LoadingBlockProps = {
  text?: string;
};

export function LoadingBlock({ text = "正在加载..." }: LoadingBlockProps) {
  return (
    <div className="loading-block">
      <span className="button-spinner" aria-hidden="true" />
      <strong>{text}</strong>
    </div>
  );
}

