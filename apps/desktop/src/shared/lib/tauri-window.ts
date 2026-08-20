import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { MouseEvent } from "react";

export async function closeApplicationWindow() {
  if (!isTauri()) {
    return;
  }

  await getCurrentWindow().close();
}

export async function startApplicationWindowDrag(event: MouseEvent<HTMLDivElement>) {
  if (!isTauri() || event.button !== 0) {
    return;
  }

  if ((event.target as HTMLElement).closest("button")) {
    return;
  }

  await getCurrentWindow().startDragging();
}
