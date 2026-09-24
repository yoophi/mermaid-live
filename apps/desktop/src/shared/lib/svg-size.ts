export interface Size {
  width: number;
  height: number;
}

function readPositiveNumber(value: string | null) {
  const parsed = parseFloat(value ?? "");
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

/**
 * The diagram's own size, preferring `viewBox` because Mermaid also writes a
 * capped inline `max-width` that does not describe the drawing.
 */
export function readSvgBaseSize(svgEl: SVGSVGElement): Size | null {
  const viewBox = svgEl.viewBox.baseVal;
  const viewBoxSize =
    viewBox.width > 0 && viewBox.height > 0 ? { width: viewBox.width, height: viewBox.height } : null;

  if (viewBoxSize) {
    return viewBoxSize;
  }

  const width = readPositiveNumber(svgEl.getAttribute("width"));
  const height = readPositiveNumber(svgEl.getAttribute("height"));

  if (width && height) {
    return { width, height };
  }

  try {
    const box = svgEl.getBBox();
    if (box.width > 0 && box.height > 0) {
      return { width: box.width, height: box.height };
    }
  } catch {
    // Some SVGs cannot provide a bbox until fully attached and laid out.
  }

  return null;
}
