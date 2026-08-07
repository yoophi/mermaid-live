import { describe, expect, it } from "vitest";
import {
  applyDocumentSnapshot,
  createChartDocument,
  documentWindowTitle,
  editDocumentSource,
  isDocumentDirty,
  type DiagramFileSnapshot,
} from "./chart-document";

const snapshot: DiagramFileSnapshot = {
  source: "flowchart LR\n  A --> B",
  binding: {
    path: "/tmp/diagram.mmd",
    fileName: "diagram.mmd",
    extension: "mmd",
    revision: { contentHash: "abc", byteLength: 20, modifiedAt: 1 },
  },
};

describe("chart document", () => {
  it("tracks dirty state against the last baseline", () => {
    const initial = createChartDocument("flowchart LR");
    const edited = editDocumentSource(initial, "flowchart TD");

    expect(isDocumentDirty(initial)).toBe(false);
    expect(isDocumentDirty(edited)).toBe(true);
    expect(isDocumentDirty(editDocumentSource(edited, initial.baselineSource))).toBe(false);
  });

  it("applies a file snapshot atomically as a clean document", () => {
    const dirty = editDocumentSource(createChartDocument("old"), "edited");
    const opened = applyDocumentSnapshot(dirty, snapshot);

    expect(opened.source).toBe(snapshot.source);
    expect(opened.baselineSource).toBe(snapshot.source);
    expect(opened.binding).toEqual(snapshot.binding);
    expect(isDocumentDirty(opened)).toBe(false);
  });

  it("derives untitled, bound, and dirty window titles", () => {
    const untitled = createChartDocument("flowchart LR");
    expect(documentWindowTitle(untitled)).toBe("제목 없음 — Mermaid Live");

    const opened = applyDocumentSnapshot(untitled, snapshot);
    expect(documentWindowTitle(opened)).toBe("diagram.mmd — Mermaid Live");
    expect(documentWindowTitle(editDocumentSource(opened, "changed"))).toBe(
      "● diagram.mmd — Mermaid Live",
    );
  });
});
