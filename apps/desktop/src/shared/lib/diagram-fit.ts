import type { Size } from "./svg-size";

export const MIN_ZOOM = 0.02;
export const MAX_ZOOM = 4;

export function clampZoom(value: number) {
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, value));
}

/**
 * The scale that makes the diagram fit an area without distorting it.
 *
 * Shared so the terminal image and the on-screen preview size a diagram by the
 * same rule; callers subtract their own padding from `available` first.
 */
export function computeFitScale(base: Size, available: Size) {
  return clampZoom(
    Math.min(Math.max(1, available.width) / base.width, Math.max(1, available.height) / base.height),
  );
}
