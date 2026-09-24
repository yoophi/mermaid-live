import type { MermaidConfig } from "mermaid";

/**
 * The one definition of how a chart looks, shared by the on-screen preview and
 * the terminal rasteriser so both paths render the same diagram.
 */
export const mermaidPreviewConfig: MermaidConfig = {
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
};

/**
 * Rasterising draws the SVG through an `<img>` onto a canvas, and
 * `foreignObject` content does not render in that context. Flowchart labels
 * must therefore be drawn as SVG text rather than HTML, which is the one
 * intended visual difference from the preview.
 */
export const mermaidRasterConfig: MermaidConfig = {
  ...mermaidPreviewConfig,
  flowchart: { htmlLabels: false },
};
