import { useEffect, useRef, useState } from "react";
import { DiagramCodeEditor } from "@/features/edit-diagram/ui/diagram-code-editor";
import { MermaidPreview } from "@/features/preview-diagram/ui/mermaid-preview";

interface DiagramWorkspaceProps {
  source: string;
  onSourceChange: (source: string) => void;
}

const SOURCE_PANE_STORAGE_KEY = "mermaid-live:source-pane-width";
const DEFAULT_SOURCE_PANE_WIDTH = 400;
const MIN_SOURCE_PANE_WIDTH = 180;
const MAX_SOURCE_PANE_WIDTH = 720;
const SOURCE_PANE_STEP = 16;

interface ResizeDragState {
  pointerX: number;
  width: number;
}

function clampSourcePaneWidth(width: number) {
  return Math.min(MAX_SOURCE_PANE_WIDTH, Math.max(MIN_SOURCE_PANE_WIDTH, width));
}

function readStoredSourcePaneWidth() {
  const stored = window.localStorage.getItem(SOURCE_PANE_STORAGE_KEY);
  const parsed = stored ? Number(stored) : DEFAULT_SOURCE_PANE_WIDTH;

  if (!Number.isFinite(parsed)) {
    return DEFAULT_SOURCE_PANE_WIDTH;
  }

  return clampSourcePaneWidth(parsed);
}

export function DiagramWorkspace({ source, onSourceChange }: DiagramWorkspaceProps) {
  const [sourcePaneWidth, setSourcePaneWidth] = useState(readStoredSourcePaneWidth);
  const resizeDragRef = useRef<ResizeDragState | null>(null);

  useEffect(() => {
    window.localStorage.setItem(SOURCE_PANE_STORAGE_KEY, String(sourcePaneWidth));
  }, [sourcePaneWidth]);

  function updateSourcePaneWidth(width: number) {
    setSourcePaneWidth(clampSourcePaneWidth(width));
  }

  function handleResizePointerDown(event: React.PointerEvent<HTMLDivElement>) {
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    resizeDragRef.current = {
      pointerX: event.clientX,
      width: sourcePaneWidth,
    };
  }

  function handleResizePointerMove(event: React.PointerEvent<HTMLDivElement>) {
    const dragState = resizeDragRef.current;
    if (!dragState) {
      return;
    }

    updateSourcePaneWidth(dragState.width + event.clientX - dragState.pointerX);
  }

  function handleResizePointerEnd(event: React.PointerEvent<HTMLDivElement>) {
    resizeDragRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  }

  function handleResizeKeyDown(event: React.KeyboardEvent<HTMLDivElement>) {
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      updateSourcePaneWidth(sourcePaneWidth - SOURCE_PANE_STEP);
    }

    if (event.key === "ArrowRight") {
      event.preventDefault();
      updateSourcePaneWidth(sourcePaneWidth + SOURCE_PANE_STEP);
    }

    if (event.key === "Home") {
      event.preventDefault();
      updateSourcePaneWidth(MIN_SOURCE_PANE_WIDTH);
    }

    if (event.key === "End") {
      event.preventDefault();
      updateSourcePaneWidth(MAX_SOURCE_PANE_WIDTH);
    }
  }

  return (
    <main
      className="grid h-screen min-h-0 overflow-hidden"
      style={{ gridTemplateColumns: `${sourcePaneWidth}px 8px minmax(0, 1fr)` }}
    >
      <section className="flex min-h-0 min-w-0 flex-col bg-card/88">
        <div className="min-h-0 flex-1">
          <DiagramCodeEditor value={source} onChange={onSourceChange} />
        </div>
      </section>

      <div
        aria-label="Resize source editor"
        aria-orientation="vertical"
        aria-valuemax={MAX_SOURCE_PANE_WIDTH}
        aria-valuemin={MIN_SOURCE_PANE_WIDTH}
        aria-valuenow={sourcePaneWidth}
        className="group flex h-full cursor-col-resize touch-none items-stretch justify-center border-x bg-border/35 outline-none transition-colors hover:bg-primary/18 focus-visible:bg-primary/18"
        onKeyDown={handleResizeKeyDown}
        onPointerCancel={handleResizePointerEnd}
        onPointerDown={handleResizePointerDown}
        onPointerMove={handleResizePointerMove}
        onPointerUp={handleResizePointerEnd}
        role="separator"
        tabIndex={0}
      >
        <div className="my-auto h-10 w-px rounded-full bg-muted-foreground/45 transition-colors group-hover:bg-primary group-focus-visible:bg-primary" />
      </div>

      <section className="flex min-h-0 min-w-0 flex-col bg-background/82">
        <div className="min-h-0 flex-1">
          <MermaidPreview source={source} />
        </div>
      </section>
    </main>
  );
}
