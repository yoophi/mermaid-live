export interface FileRevision {
  contentHash: string;
  byteLength: number;
  modifiedAt: number | null;
}

export interface DocumentFileBinding {
  path: string;
  fileName: string;
  extension: "mmd" | "mermaid";
  revision: FileRevision;
}

export interface DiagramFileSnapshot {
  source: string;
  binding: DocumentFileBinding;
}

export interface ChartDocument {
  source: string;
  baselineSource: string;
  binding: DocumentFileBinding | null;
}

export function createChartDocument(source: string): ChartDocument {
  return { source, baselineSource: source, binding: null };
}

export function editDocumentSource(document: ChartDocument, source: string): ChartDocument {
  return { ...document, source };
}

export function replaceUntitledSource(source: string): ChartDocument {
  return createChartDocument(source);
}

export function applyDocumentSnapshot(
  _document: ChartDocument,
  snapshot: DiagramFileSnapshot,
): ChartDocument {
  return {
    source: snapshot.source,
    baselineSource: snapshot.source,
    binding: snapshot.binding,
  };
}

export function isDocumentDirty(document: ChartDocument): boolean {
  return document.source !== document.baselineSource;
}

export function documentDisplayName(document: ChartDocument): string {
  return document.binding?.fileName ?? "제목 없음";
}

export function documentWindowTitle(document: ChartDocument): string {
  const dirtyPrefix = isDocumentDirty(document) ? "● " : "";
  return `${dirtyPrefix}${documentDisplayName(document)} — Mermaid Live`;
}
