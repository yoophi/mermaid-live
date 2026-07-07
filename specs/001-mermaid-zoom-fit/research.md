# Research: Mermaid Zoom Fit

## Decision: Use AW's SVG fit model as the reference behavior

**Rationale**: AW's Mermaid fit mode relies on the rendered SVG's own `viewBox` and aspect-ratio behavior. In fit mode, the SVG is placed in a full-size parent, centered, and sized with `height: 100%`, `width: 100%`, and `max-width: 100% !important` to override Mermaid's inline max-width. This matches the desired outcome: preserve the current display surface while letting the diagram fit the available area.

**Alternatives considered**:

- Add a full-screen modal like AW's agent-run expanded view. Rejected because the specification explicitly requires keeping the current display method.
- Keep only the existing manual "fit to window" button. Rejected because the feature requires automatic best-size rendering when the chart appears.

## Decision: Keep the existing MermaidPreview zoom/pan component

**Rationale**: `apps/desktop/src/features/preview-diagram/ui/mermaid-preview.tsx` already renders Mermaid, tracks `zoom`, tracks `pan`, measures SVG base size, blocks native pinch zoom, and exposes zoom in/out plus fit controls. The least disruptive implementation is to improve its automatic fit lifecycle rather than replace it.

**Alternatives considered**:

- Introduce a new generic fit container in `shared`. Rejected for now because only one feature needs the behavior and the current component already owns the workflow state.
- Move Mermaid rendering into `entities/diagram`. Rejected because the current change is an interaction/display workflow, not a domain entity rule.

## Decision: Recalculate fit after render and preview-area resize

**Rationale**: Initial rendering and container resizing are the two moments where a previously correct zoom can become wrong. The plan should ensure the preview measures the rendered SVG after Mermaid has produced DOM output, then computes a fit zoom against the current viewport. It should also respond when the preview area changes size.

**Alternatives considered**:

- Recalculate only when the source changes. Rejected because the spec requires handling window and panel size changes.
- Recalculate continuously on every render. Rejected because it can fight user-controlled zoom/pan and create unnecessary updates.

## Decision: Use SVG dimensions from viewBox first

**Rationale**: Mermaid SVGs commonly provide a `viewBox`, which is the most stable intrinsic coordinate space for fit calculations. Width and height attributes are useful fallbacks when a valid viewBox is absent.

**Alternatives considered**:

- Use `getBoundingClientRect()` on the scaled visible SVG. Rejected because it can reflect current zoomed dimensions rather than the intrinsic base size.
- Parse the SVG string manually. Rejected because the DOM API is already available after rendering and is less brittle.

## Decision: Keep transform panning, but do not rely on transform scale for fit layout

**Rationale**: AW's expanded view notes that transform scaling does not change layout size, which can prevent overflow containers from scrolling correctly when zoom exceeds 100%. The current Mermaid Live preview expresses zoom through the content box width and height, with transform reserved for centering and pan translation. That approach should remain compatible with fit sizing.

**Alternatives considered**:

- Switch to `transform: scale(...)` for zoom. Rejected because it risks the overflow and clipping problems AW avoided.
- Remove pan support. Rejected because it is existing user-facing behavior and still useful after manual zoom.

## Decision: No native or persistence changes

**Rationale**: The feature is entirely about how the already-rendered chart appears in the preview surface. No filesystem, persistence, process, or OS integration is involved.

**Alternatives considered**:

- Persist a user's last zoom level. Rejected because automatic fit is the requested behavior and persistence would add unrequested state.
