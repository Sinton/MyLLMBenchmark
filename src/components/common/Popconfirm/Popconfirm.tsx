import type { ReactNode } from "react";
import { Button } from "../Button";
import { Popover } from "../Popover";

type PopconfirmProps = {
  title: string;
  description?: string;
  confirmText?: string;
  cancelText?: string;
  onConfirm: () => void;
  children: ReactNode;
};

export function Popconfirm({
  title,
  description,
  confirmText = "确认",
  cancelText = "取消",
  onConfirm,
  children,
}: PopconfirmProps) {
  return (
    <Popover
      className="popconfirm"
      trigger={({ toggle }) => (
        <span onClick={toggle} role="presentation">
          {children}
        </span>
      )}
    >
      {({ close }) => (
      <div className="popconfirm-panel">
        <strong>{title}</strong>
        {description && <span>{description}</span>}
        <div className="popconfirm-actions">
          <Button onClick={close} variant="ghost">
            {cancelText}
          </Button>
          <Button
            onClick={() => {
              onConfirm();
              close();
            }}
            variant="primary"
          >
            {confirmText}
          </Button>
        </div>
      </div>
      )}
    </Popover>
  );
}

