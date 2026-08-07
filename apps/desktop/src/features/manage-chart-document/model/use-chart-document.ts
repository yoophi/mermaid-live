import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  applyDocumentSnapshot,
  createChartDocument,
  documentDisplayName,
  documentWindowTitle,
  editDocumentSource,
  isDocumentDirty,
  replaceUntitledSource,
  type ChartDocument,
} from "@/entities/chart-document";
import {
  authorizeWindowClose,
  openDiagramFile,
  promptExternalConflict,
  promptUnsavedChanges,
  saveDiagramFile,
  showDocumentError,
} from "../api/chart-file";

const OPEN_EVENT = "open-chart-document-request";
const SAVE_EVENT = "save-chart-document-request";
const SAVE_AS_EVENT = "save-chart-document-as-request";
const CLOSE_EVENT = "close-chart-document-request";

export function shouldUpdateDocumentWindowTitle(
  previousTitle: string | null,
  document: ChartDocument,
) {
  return previousTitle !== documentWindowTitle(document);
}

function defaultDiagramFileName() {
  const now = new Date();
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}.mmd`;
}

export function useChartDocument(initialSource: string) {
  const [document, setDocumentState] = useState(() => createChartDocument(initialSource));
  const documentRef = useRef(document);
  const operationRef = useRef(Promise.resolve());
  const windowTitleRef = useRef<string | null>(null);

  const setDocument = useCallback((next: ChartDocument) => {
    documentRef.current = next;
    setDocumentState(next);
  }, []);

  const enqueue = useCallback((operation: () => Promise<void>) => {
    operationRef.current = operationRef.current.then(operation, operation);
  }, []);

  const save = useCallback(async (saveAs = false, force = false): Promise<boolean> => {
    const current = documentRef.current;
    try {
      const outcome = await saveDiagramFile({
        source: current.source,
        targetPath: saveAs ? null : current.binding?.path ?? null,
        expectedRevision: saveAs ? null : current.binding?.revision ?? null,
        force,
        suggestedFileName: current.binding?.fileName ?? defaultDiagramFileName(),
      });
      if (outcome.status === "cancelled") return false;
      if (outcome.status === "saved") {
        setDocument(applyDocumentSnapshot(current, outcome.snapshot));
        return true;
      }

      const decision = await promptExternalConflict(documentDisplayName(current));
      if (decision === "reload") {
        setDocument(applyDocumentSnapshot(current, outcome.diskSnapshot));
        return true;
      }
      if (decision === "overwrite") return save(false, true);
      return false;
    } catch (error) {
      await showDocumentError("파일을 저장할 수 없음", error);
      return false;
    }
  }, [setDocument]);

  const confirmDestructiveIntent = useCallback(async () => {
    const current = documentRef.current;
    if (!isDocumentDirty(current)) return true;
    const decision = await promptUnsavedChanges(documentDisplayName(current));
    if (decision === "cancel") return false;
    if (decision === "discard") return true;
    return save(false);
  }, [save]);

  const open = useCallback(async () => {
    if (!(await confirmDestructiveIntent())) return;
    try {
      const outcome = await openDiagramFile();
      if (outcome.status === "opened") {
        setDocument(applyDocumentSnapshot(documentRef.current, outcome.snapshot));
      }
    } catch (error) {
      await showDocumentError("파일을 열 수 없음", error);
    }
  }, [confirmDestructiveIntent, setDocument]);

  const close = useCallback(async () => {
    if (await confirmDestructiveIntent()) {
      await authorizeWindowClose();
    }
  }, [confirmDestructiveIntent]);

  useEffect(() => {
    if (!shouldUpdateDocumentWindowTitle(windowTitleRef.current, document)) return;
    const title = documentWindowTitle(document);
    windowTitleRef.current = title;
    void getCurrentWindow().setTitle(title);
  }, [document]);

  useEffect(() => {
    const cleanups: Array<() => void> = [];
    let disposed = false;
    void Promise.all([
      listen(OPEN_EVENT, () => enqueue(open)),
      listen(SAVE_EVENT, () => enqueue(async () => { await save(false); })),
      listen(SAVE_AS_EVENT, () => enqueue(async () => { await save(true); })),
      listen(CLOSE_EVENT, () => enqueue(close)),
    ]).then((unlisteners) => {
      if (disposed) {
        unlisteners.forEach((cleanup) => cleanup());
      } else {
        cleanups.push(...unlisteners);
      }
    });
    return () => {
      disposed = true;
      cleanups.forEach((cleanup) => cleanup());
    };
  }, [close, enqueue, open, save]);

  const editSource = useCallback(
    (source: string) => setDocument(editDocumentSource(documentRef.current, source)),
    [setDocument],
  );
  const loadUntitledSource = useCallback(
    (source: string) => setDocument(replaceUntitledSource(source)),
    [setDocument],
  );

  return {
    document,
    editSource,
    loadUntitledSource,
  };
}
