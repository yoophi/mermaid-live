import { Frame, TitleBar } from "@react95/core";
import { Logo } from "@react95/icons/Logo";
import { useEffect, useRef, type KeyboardEvent, type MouseEvent } from "react";
import { closeApplicationWindow, startApplicationWindowDrag } from "@/shared/lib/tauri-window";
import { Win95Button } from "@/shared/ui/window-95-controls";

const ABOUT_WINDOW_TITLE_ID = "about-mermaid-live-title";
const ABOUT_WINDOW_DESCRIPTION_ID = "about-mermaid-live-description";

export function AboutWindow() {
  const primaryButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    primaryButtonRef.current?.focus();
  }, []);

  function closeWindow() {
    void closeApplicationWindow();
  }

  function handleKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (event.key !== "Escape") {
      return;
    }

    event.preventDefault();
    closeWindow();
  }

  function handleTitleBarMouseDown(event: MouseEvent<HTMLDivElement>) {
    void startApplicationWindowDrag(event);
  }

  return (
    <main className="about-window-95__stage" onKeyDown={handleKeyDown}>
      <Frame
        aria-describedby={ABOUT_WINDOW_DESCRIPTION_ID}
        aria-labelledby={ABOUT_WINDOW_TITLE_ID}
        className="about-window-95"
        role="dialog"
      >
        <TitleBar
          active
          className="about-window-95__title-bar"
          data-tauri-drag-region
          icon={
            <Logo
              aria-hidden="true"
              className="about-window-95__title-icon"
              focusable="false"
              variant="16x16_4"
            />
          }
          onMouseDown={handleTitleBarMouseDown}
          title="About Mermaid Live"
        >
          <TitleBar.OptionsBox className="about-window-95__title-controls">
            <TitleBar.Close
              aria-label="Close About window"
              onClick={closeWindow}
              onMouseDown={(event) => event.stopPropagation()}
              title="Close"
            />
          </TitleBar.OptionsBox>
        </TitleBar>

        <div className="about-window-95__body">
          <div className="about-window-95__summary">
            <Logo
              aria-hidden="true"
              className="about-window-95__product-icon"
              focusable="false"
              variant="32x32_4"
            />
            <div>
              <h1 id={ABOUT_WINDOW_TITLE_ID} className="about-window-95__product-name">
                Mermaid Live
              </h1>
              <p className="about-window-95__version">
                Version {__MERMAID_LIVE_BUILD_VERSION__}
              </p>
              <p id={ABOUT_WINDOW_DESCRIPTION_ID} className="about-window-95__description">
                Mermaid diagram editor for the desktop.
              </p>
            </div>
          </div>

          <div aria-hidden="true" className="about-window-95__separator" />

          <p className="about-window-95__copyright">© 2026 yoophi</p>

          <div className="about-window-95__actions">
            <Win95Button
              ref={primaryButtonRef}
              className="about-window-95__ok-button"
              onClick={closeWindow}
              type="button"
            >
              OK
            </Win95Button>
          </div>
        </div>
      </Frame>
    </main>
  );
}
