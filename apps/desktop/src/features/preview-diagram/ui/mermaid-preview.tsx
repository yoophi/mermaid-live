import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
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

const MIN_ZOOM = 0.02;
const MAX_ZOOM = 4;
const ZOOM_STEP = 0.2;
const FIT_PADDING = 48;

interface PanPosition {
  x: number;
  y: number;
}

interface Size {
  width: number;
  height: number;
}

function clampZoom(value: number) {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, value));
}

function readSvgBaseSize(svgEl: SVGSVGElement): Size | null {
  const viewBox = svgEl.viewBox.baseVal;
  const width = viewBox.width > 0 ? viewBox.width : parseFloat(svgEl.getAttribute("width") ?? "");
  const height =
    viewBox.height > 0 ? viewBox.height : parseFloat(svgEl.getAttribute("height") ?? "");

  if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) {
    return null;
  }

  return { width, height };
}

function getElementSize(el: HTMLElement | null): Size | null {
  if (!el) {
    return null;
  }

  const rect = el.getBoundingClientRect();
  if (rect.width <= 0 || rect.height <= 0) {
    return null;
  }

  return { width: rect.width, height: rect.height };
}

function getFitZoom(baseSize: Size | null, viewportEl: HTMLElement | null) {
  const viewportSize = getElementSize(viewportEl);
  if (!baseSize || !viewportSize) {
    return 1;
  }

  const availableWidth = Math.max(1, viewportSize.width - FIT_PADDING * 2);
  const availableHeight = Math.max(1, viewportSize.height - FIT_PADDING * 2);

  return clampZoom(Math.min(availableWidth / baseSize.width, availableHeight / baseSize.height));
}

export function MermaidPreview({ source }: MermaidPreviewProps) {
  const id = useId().replace(/:/g, "");
  const viewportRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const dragStartRef = useRef<PanPosition | null>(null);
  const baseSizeRef = useRef<Size | null>(null);
  const [svg, setSvg] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [baseSize, setBaseSize] = useState<Size | null>(null);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState<PanPosition>({ x: 0, y: 0 });

  function measureBaseSize() {
    const svgEl = contentRef.current?.querySelector("svg");
    if (!svgEl) {
      return null;
    }

    const nextBaseSize = readSvgBaseSize(svgEl);
    if (nextBaseSize) {
      baseSizeRef.current = nextBaseSize;
    }
    return nextBaseSize;
  }

  useLayoutEffect(() => {
    if (!svg) {
      baseSizeRef.current = null;
      setBaseSize(null);
      return;
    }

    const nextBaseSize = measureBaseSize();
    if (!nextBaseSize) {
      return;
    }

    const initialZoom = getFitZoom(nextBaseSize, viewportRef.current);
    setBaseSize((current) =>
      current?.width === nextBaseSize.width && current.height === nextBaseSize.height
        ? current
        : nextBaseSize,
    );
    setZoom((current) => (Math.abs(current - initialZoom) < 0.001 ? current : initialZoom));
    setPan((current) => (current.x === 0 && current.y === 0 ? current : { x: 0, y: 0 }));
  }, [svg]);

  useEffect(() => {
    let cancelled = false;

    async function renderDiagram() {
      try {
        const normalizedSource = source.trim();

        if (!normalizedSource) {
          setSvg("");
          setError(null);
          baseSizeRef.current = null;
          setBaseSize(null);
          setZoom(1);
          setPan({ x: 0, y: 0 });
          return;
        }

        const result = await mermaid.render(`diagram-${id}`, normalizedSource);

        if (!cancelled) {
          baseSizeRef.current = null;
          setBaseSize(null);
          setSvg(result.svg);
          setError(null);
          setPan({ x: 0, y: 0 });
        }
      } catch (err) {
        if (!cancelled) {
          baseSizeRef.current = null;
          setBaseSize(null);
          setError(err instanceof Error ? err.message : "Invalid Mermaid syntax");
        }
      }
    }

    renderDiagram();

    return () => {
      cancelled = true;
    };
  }, [source]);

  // Trackpad pinch is noisy in the webview; block native pinch/page zoom here.
  useEffect(() => {
    const el = viewportRef.current;
    if (!el) {
      return;
    }

    function preventPinchWheel(event: WheelEvent) {
      if (event.ctrlKey || event.metaKey) {
        event.preventDefault();
      }
    }

    function preventGesture(event: Event) {
      event.preventDefault();
    }

    el.addEventListener("wheel", preventPinchWheel, { passive: false });
    el.addEventListener("gesturestart", preventGesture, { passive: false });
    el.addEventListener("gesturechange", preventGesture, { passive: false });
    el.addEventListener("gestureend", preventGesture, { passive: false });

    return () => {
      el.removeEventListener("wheel", preventPinchWheel);
      el.removeEventListener("gesturestart", preventGesture);
      el.removeEventListener("gesturechange", preventGesture);
      el.removeEventListener("gestureend", preventGesture);
    };
  }, []);

  function changeZoom(delta: number) {
    setZoom((current) => clampZoom(current + delta));
  }

  function fitToWindow() {
    const currentBaseSize = measureBaseSize() ?? baseSizeRef.current ?? baseSize;
    if (!currentBaseSize) {
      return;
    }

    const nextZoom = getFitZoom(currentBaseSize, viewportRef.current);
    setZoom(nextZoom);
    setPan({ x: 0, y: 0 });
  }

  function handlePointerDown(event: React.PointerEvent<HTMLDivElement>) {
    if (!svg) {
      return;
    }

    event.currentTarget.setPointerCapture(event.pointerId);
    dragStartRef.current = {
      x: event.clientX - pan.x,
      y: event.clientY - pan.y,
    };
    contentRef.current?.style.setProperty("cursor", "grabbing");
  }

  function handlePointerMove(event: React.PointerEvent<HTMLDivElement>) {
    const dragStart = dragStartRef.current;
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
    dragStartRef.current = null;
    contentRef.current?.style.setProperty("cursor", "grab");
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
          aria-label="Fit diagram to window"
          disabled={!svg}
          onClick={fitToWindow}
          size="icon"
          title="Fit to window"
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
            ref={contentRef}
            className="absolute left-1/2 top-1/2 will-change-transform"
            dangerouslySetInnerHTML={{ __html: svg }}
            style={{
              cursor: "grab",
              height: baseSize ? `${baseSize.height * zoom}px` : undefined,
              transform: `translate(-50%, -50%) translate(${pan.x}px, ${pan.y}px)`,
              transformOrigin: "center",
              visibility: baseSize ? "visible" : "hidden",
              width: baseSize ? `${baseSize.width * zoom}px` : undefined,
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
