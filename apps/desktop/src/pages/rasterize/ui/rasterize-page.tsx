import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import mermaid from "mermaid";
import { computeFitScale } from "@/shared/lib/diagram-fit";
import { mermaidRasterConfig } from "@/shared/lib/mermaid-config";
import { readSvgBaseSize } from "@/shared/lib/svg-size";

/**
 * Charts use dark text on light shapes, so the image needs an opaque light
 * background to stay readable in a terminal of any theme.
 */
const RASTER_BACKGROUND = "#ffffff";

interface RasterRequest {
  requestId: string;
  sourceFile: string;
  /** Maximum output size in device pixels; already includes `scale`. */
  widthPx: number;
  heightPx: number;
  scale: number;
}

function readRequest(search: string): RasterRequest | null {
  const params = new URLSearchParams(search);
  const requestId = params.get("requestId");
  const sourceFile = params.get("sourceFile");
  const widthPx = Number(params.get("w"));
  const heightPx = Number(params.get("h"));
  const scale = Number(params.get("scale"));

  if (
    !requestId ||
    !sourceFile ||
    !Number.isFinite(widthPx) ||
    !Number.isFinite(heightPx) ||
    widthPx <= 0 ||
    heightPx <= 0
  ) {
    return null;
  }

  return {
    requestId,
    sourceFile,
    widthPx,
    heightPx,
    scale: Number.isFinite(scale) && scale > 0 ? scale : 1,
  };
}

/** Measures the diagram off-screen, where a bbox is available if needed. */
function withAttachedSvg<T>(svgMarkup: string, use: (svgEl: SVGSVGElement) => T): T {
  const holder = document.createElement("div");
  holder.setAttribute("aria-hidden", "true");
  holder.style.position = "fixed";
  holder.style.left = "-10000px";
  holder.style.top = "0";
  holder.innerHTML = svgMarkup;
  document.body.appendChild(holder);

  try {
    const svgEl = holder.querySelector("svg");
    if (!svgEl) {
      throw new Error("Mermaid produced no SVG");
    }
    return use(svgEl as SVGSVGElement);
  } finally {
    holder.remove();
  }
}

function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error("the rendered SVG could not be loaded as an image"));
    image.src = url;
  });
}

async function rasterize(request: RasterRequest): Promise<string> {
  mermaid.initialize(mermaidRasterConfig);

  const source = await invoke<string>("read_diagram_file", { path: request.sourceFile });
  const { svg } = await mermaid.render(`raster-${request.requestId}`, source.trim());

  const { markup, base } = withAttachedSvg(svg, (svgEl) => {
    const measured = readSvgBaseSize(svgEl);
    if (!measured) {
      throw new Error("the diagram reported no usable size");
    }

    // Mermaid caps the SVG with an inline max-width; pin the real size so the
    // image element rasterises the whole drawing.
    svgEl.setAttribute("width", String(measured.width));
    svgEl.setAttribute("height", String(measured.height));
    svgEl.style.removeProperty("max-width");

    return { markup: new XMLSerializer().serializeToString(svgEl), base: measured };
  });

  // Fit in logical pixels, then emit at `scale` so the image stays crisp
  // without occupying more cells than the terminal offered.
  const fit = computeFitScale(base, {
    width: request.widthPx / request.scale,
    height: request.heightPx / request.scale,
  });

  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(base.width * fit * request.scale));
  canvas.height = Math.max(1, Math.round(base.height * fit * request.scale));

  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("this webview provides no 2D canvas");
  }

  // A data URL, not a blob URL: WebKit treats a canvas as tainted once a
  // blob-sourced SVG is drawn on it, and toDataURL then fails with
  // "The operation is insecure". This only shows up at runtime.
  const image = await loadImage(
    `data:image/svg+xml;charset=utf-8,${encodeURIComponent(markup)}`,
  );
  context.fillStyle = RASTER_BACKGROUND;
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.drawImage(image, 0, 0, canvas.width, canvas.height);

  return canvas.toDataURL("image/png").replace(/^data:image\/png;base64,/, "");
}

/**
 * Invisible surface that turns one chart into one PNG and hands it back.
 *
 * The desktop app opens this in a hidden window so the terminal path reuses the
 * same Mermaid version and theme as the on-screen preview.
 */
export function RasterizePage() {
  useEffect(() => {
    const request = readRequest(window.location.search);
    if (!request) {
      return;
    }

    let cancelled = false;

    rasterize(request)
      .then((pngBase64) => {
        if (!cancelled) {
          return invoke("deliver_chart_png", { requestId: request.requestId, pngBase64 });
        }
      })
      .catch((error: unknown) => {
        if (cancelled) {
          return;
        }
        return invoke("deliver_chart_png", {
          requestId: request.requestId,
          error: error instanceof Error ? error.message : String(error),
        });
      })
      .catch((error: unknown) => {
        console.error("Failed to deliver the rendered chart", error);
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return null;
}
