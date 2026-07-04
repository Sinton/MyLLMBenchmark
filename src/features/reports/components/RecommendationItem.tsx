import type { ReactNode } from "react";

type RecommendationItemProps = {
  icon: ReactNode;
  title: string;
  text: string;
};

export function RecommendationItem({ icon, title, text }: RecommendationItemProps) {
  return (
    <div className="recommendation-item">
      <span>{icon}</span>
      <strong>{title}</strong>
      <p>{text}</p>
    </div>
  );
}
