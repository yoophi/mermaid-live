import { useEffect, useId, useRef, useState } from "react";
import { Maximize2, Minus, Plus } from "lucide-react";
import mermaid from "mermaid";
import { Button } from "@/shared/ui/button";

mermaid.initialize({
  startOnLoad: false,
  securityLevel: "strict",
  theme: "base",
  themeVariables: {
    background: "transparent",
    primaryColor: "#f7e0a4",
    primaryTextColor: "#1f2933",
    primaryBorderColor: "#41616f",
    lineColor: "#41616f",
    fontFamily: "Avenir Next, Segoe UI, sans-serif",
  },
});

interface MermaidPreviewProps {
  source: string;
}

const MIN_ZOOM = 0.25;
const MAX_ZOOM = 4;
const ZOOM_STEP = 0.2;

interface PanPosition {
  x: number;
  y: number;
}

function clampZoom(value: number) {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, value));
}

export function MermaidPreview({ source }: MermaidPreviewProps) {
  const id = useId().replace(/:/g, "");
  const viewportRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef("");
  const [svg, setSvg] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState<PanPosition>({ x: 0, y: 0 });
  const [dragStart, setDragStart] = useState<PanPosition | null>(null);

  useEffect(() => {
    svgRef.current = svg;
  }, [svg]);

  useEffect(() => {
    let cancelled = false;

    async function renderDiagram() {
      try {
        const normalizedSource = source.trim();

        if (!normalizedSource) {
          setSvg("");
          setError(null);
          setZoom(1);
          setPan({ x: 0, y: 0 });
          return;
        }

        const result = await mermaid.render(`diagram-${id}`, normalizedSource);

        if (!cancelled) {
          setSvg(result.svg);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Invalid Mermaid syntax");
        }
      }
    }

    renderDiagram();

    return () => {
      cancelled = true;
    };
  }, [source]);

  // React's synthetic wheel listener is passive; bind natively so preventDefault works.
  useEffect(() => {
    const el = viewportRef.current;
    if (!el) {
      return;
    }

    function handleWheel(event: WheelEvent) {
      if (!svgRef.current) {
        return;
      }
      event.preventDefault();
      const direction = event.deltaY > 0 ? -1 : 1;
      setZoom((current) => clampZoom(current + direction * ZOOM_STEP));
    }

    el.addEventListener("wheel", handleWheel, { passive: false });
    return () => el.removeEventListener("wheel", handleWheel);
  }, []);

  function changeZoom(delta: number) {
    setZoom((current) => clampZoom(current + delta));
  }

  function resetViewport() {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  }

  function handlePointerDown(event: React.PointerEvent<HTMLDivElement>) {
    if (!svg) {
      return;
    }

    event.currentTarget.setPointerCapture(event.pointerId);
    setDragStart({
      x: event.clientX - pan.x,
      y: event.clientY - pan.y,
    });
  }

  function handlePointerMove(event: React.PointerEvent<HTMLDivElement>) {
    if (!dragStart) {
      return;
    }

    const nextX = event.clientX - dragStart.x;
    const nextY = event.clientY - dragStart.y;

    setPan((current) =>
      current.x === nextX && current.y === nextY ? current : { x: nextX, y: nextY },
    );
  }

  function handlePointerEnd() {
    setDragStart(null);
  }

  if (error) {
    return (
      <div className="grid h-full place-items-center p-8">
        <div className="max-w-xl rounded-md border border-destructive/35 bg-destructive/8 p-4 text-sm text-destructive">
          {error}
        </div>
      </div>
    );
  }

  return (
    <div className="relative h-full overflow-hidden">
      <div className="absolute right-4 top-4 z-10 flex items-center gap-1 rounded-md border bg-card/92 p-1 shadow-sm backdrop-blur">
        <Button
          aria-label="Zoom out"
          disabled={!svg || zoom <= MIN_ZOOM}
          onClick={() => changeZoom(-ZOOM_STEP)}
          size="icon"
          type="button"
          variant="ghost"
        >
          <Minus />
        </Button>
        <div className="min-w-12 text-center text-xs font-semibold text-muted-foreground">
          {Math.round(zoom * 100)}%
        </div>
        <Button
          aria-label="Zoom in"
          disabled={!svg || zoom >= MAX_ZOOM}
          onClick={() => changeZoom(ZOOM_STEP)}
          size="icon"
          type="button"
          variant="ghost"
        >
          <Plus />
        </Button>
        <Button
          aria-label="Reset viewport"
          disabled={!svg}
          onClick={resetViewport}
          size="icon"
          type="button"
          variant="ghost"
        >
          <Maximize2 />
        </Button>
      </div>

      <div
        ref={viewportRef}
        className="mermaid-preview h-full touch-none select-none overflow-hidden"
        onPointerCancel={handlePointerEnd}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerEnd}
      >
        {svg ? (
          <div
            className="absolute left-1/2 top-1/2 max-h-full max-w-full will-change-transform"
            dangerouslySetInnerHTML={{ __html: svg }}
            style={{
              cursor: dragStart ? "grabbing" : "grab",
              transform: `translate(-50%, -50%) translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
              transformOrigin: "center",
            }}
          />
        ) : (
          <div className="grid h-full place-items-center text-sm text-muted-foreground">
            No diagram source
          </div>
        )}
      </div>
    </div>
  );
}
