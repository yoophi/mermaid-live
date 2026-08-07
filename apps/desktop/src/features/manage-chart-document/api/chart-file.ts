import { invoke } from "@tauri-apps/api/core";
import type { DiagramFileSnapshot, FileRevision } from "@/entities/chart-document";

export type OpenDiagramFileOutcome =
  | { status: "opened"; snapshot: DiagramFileSnapshot }
  | { status: "cancelled" };

export type SaveDiagramFileOutcome =
  | { status: "saved"; snapshot: DiagramFileSnapshot }
  | { status: "conflict"; diskSnapshot: DiagramFileSnapshot }
  | { status: "cancelled" };

export interface SaveDiagramFileRequest {
  source: string;
  targetPath: string | null;
  expectedRevision: FileRevision | null;
  force: boolean;
  suggestedFileName: string;
}

export function openDiagramFile() {
  return invoke<OpenDiagramFileOutcome>("open_diagram_file");
}

export function saveDiagramFile(request: SaveDiagramFileRequest) {
  return invoke<SaveDiagramFileOutcome>("save_diagram_file", { request });
}

export function promptUnsavedChanges(fileName: string) {
  return invoke<"save" | "discard" | "cancel">("prompt_unsaved_changes", { fileName });
}

export function promptExternalConflict(fileName: string) {
  return invoke<"reload" | "overwrite" | "cancel">("prompt_external_conflict", { fileName });
}

export function authorizeWindowClose() {
  return invoke<void>("authorize_window_close");
}

export function showDocumentError(title: string, error: unknown) {
  const message = typeof error === "object" && error && "message" in error
    ? String(error.message)
    : String(error);
  return invoke<void>("show_document_error", { title, message });
}
