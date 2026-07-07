# UI Contract: Mermaid Preview Fit Behavior

## Scope

This contract defines observable behavior for the Mermaid preview surface. It does not define an external network API or native command.

## Initial Render

**Given** a non-empty Mermaid source that renders successfully  
**When** the preview displays the diagram  
**Then** the full diagram is visible within the current preview area  
**And** the diagram is centered  
**And** the current inline preview mode is preserved.

## Large Diagram

**Given** a rendered Mermaid diagram wider or taller than the preview area  
**When** automatic fit runs  
**Then** the diagram is reduced proportionally until both width and height fit within the available preview area.

## Small Diagram

**Given** a rendered Mermaid diagram smaller than the preview area  
**When** automatic fit runs  
**Then** the diagram remains readable and is not unnecessarily reduced below its natural readable size.

## Resize

**Given** a rendered Mermaid diagram  
**When** the preview area changes size  
**Then** the diagram is fitted to the new area within 1 second  
**And** the diagram remains centered after the automatic fit.

## Manual Controls

**Given** a rendered Mermaid diagram  
**When** the user selects zoom in, zoom out, pan, or fit  
**Then** the existing controls continue to work  
**And** selecting fit recenters the diagram and applies the current best-fit zoom.

## Error And Empty States

**Given** empty source or Mermaid render failure  
**When** the preview updates  
**Then** the existing empty or error state is shown  
**And** automatic fit does not hide, replace, or mask that state.
