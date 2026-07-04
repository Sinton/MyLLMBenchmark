type ProgressBarProps = {
  value: number;
  label?: string;
};

export function ProgressBar({ value, label }: ProgressBarProps) {
  const normalized = Math.max(0, Math.min(100, value));
  const filledSegments = Math.round(normalized / 5);

  return (
    <div
      aria-label={label}
      aria-valuemax={100}
      aria-valuemin={0}
      aria-valuenow={normalized}
      className="progress-bar"
      role="progressbar"
    >
      {Array.from({ length: 20 }, (_, index) => (
        <span
          aria-hidden="true"
          className={index < filledSegments ? "is-filled" : ""}
          key={index}
        />
      ))}
    </div>
  );
}
