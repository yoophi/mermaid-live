import { Frame, TitleBar } from "@react95/core";
import { Logo } from "@react95/icons/Logo";
import type { MouseEventHandler, PropsWithChildren } from "react";
import { cn } from "@/shared/lib/utils";

interface Window95Props extends PropsWithChildren {
  className?: string;
  title: string;
  onClose: () => void;
  onTitleBarMouseDown: MouseEventHandler<HTMLDivElement>;
}

export function Window95({
  children,
  className,
  title,
  onClose,
  onTitleBarMouseDown,
}: Window95Props) {
  return (
    <Frame aria-label={title} className={cn("window-95", className)} role="dialog">
      <TitleBar
        active
        className="window-95__title-bar"
        data-tauri-drag-region
        icon={
          <Logo
            aria-hidden="true"
            className="window-95__app-icon"
            focusable="false"
            variant="16x16_4"
          />
        }
        onMouseDown={onTitleBarMouseDown}
        title={title}
      >
        <TitleBar.OptionsBox className="window-95__title-controls">
          <TitleBar.Close
            aria-label="Close window"
            onClick={onClose}
            onMouseDown={(event) => event.stopPropagation()}
            title="Close"
          />
        </TitleBar.OptionsBox>
      </TitleBar>

      <div className="window-95__canvas">{children}</div>
    </Frame>
  );
}
