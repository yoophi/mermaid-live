# Data Model: Mermaid Zoom Fit

## Mermaid Chart

Represents the diagram content displayed in the preview.

**Fields**:

- `source`: Mermaid source text after trimming.
- `renderedSvg`: SVG markup returned by the Mermaid renderer.
- `renderStatus`: One of `empty`, `rendering`, `rendered`, or `failed`.
- `errorMessage`: User-facing render error when rendering fails.

**Validation rules**:

- Empty source shows the existing empty state and does not attempt fit calculation.
- Failed render shows the existing error state and does not attempt fit calculation.
- Fit calculation only runs when a rendered SVG element exists.

## Display Area

Represents the current area available for the previewed diagram.

**Fields**:

- `width`: Available preview width.
- `height`: Available preview height.
- `fitPadding`: Reserved spacing around the diagram.

**Validation rules**:

- Width and height must be positive before fit calculation.
- If dimensions are unavailable, the preview must keep a stable fallback state rather than hiding the chart permanently.

## SVG Base Size

Represents the diagram's intrinsic size before preview zoom is applied.

**Fields**:

- `width`: Intrinsic SVG width from `viewBox` or width attribute.
- `height`: Intrinsic SVG height from `viewBox` or height attribute.

**Validation rules**:

- Width and height must be finite positive numbers.
- `viewBox` dimensions take precedence over width and height attributes.

## Fit Result

Represents the calculated display state for the preview.

**Fields**:

- `zoom`: The selected fit zoom, clamped to the preview's supported zoom range.
- `panX`: Horizontal pan offset.
- `panY`: Vertical pan offset.
- `isAutoFit`: Whether the state was produced by automatic fit.

**Validation rules**:

- Zoom must stay within the configured minimum and maximum zoom values.
- Automatic fit resets pan to the centered position.
- Manual zoom and pan remain available after automatic fit.

## State Transitions

```text
empty source -> empty preview
non-empty source -> rendering
rendering -> rendered -> measure SVG base size -> compute fit result -> display centered chart
rendering -> failed -> existing error state
rendered + display area changed -> recompute fit result -> display centered chart
rendered + user zoom/pan action -> manual zoom/pan state
rendered + user fit action -> compute fit result -> display centered chart
```
