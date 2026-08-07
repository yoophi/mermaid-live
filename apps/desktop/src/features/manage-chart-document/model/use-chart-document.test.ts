import { describe, expect, it } from "vitest";
import {
  applyDocumentSnapshot,
  createChartDocument,
  editDocumentSource,
  type DiagramFileSnapshot,
} from "@/entities/chart-document";
import { shouldUpdateDocumentWindowTitle } from "./use-chart-document";

const snapshot: DiagramFileSnapshot = {
  source: "flowchart LR\n  A --> B",
  binding: {
    path: "/tmp/diagram.mmd",
    fileName: "diagram.mmd",
    extension: "mmd",
    revision: { contentHash: "abc", byteLength: 20, modifiedAt: 1 },
  },
};

describe("chart document window title synchronization", () => {
  it("updates only when the derived document title changes", () => {
    const untitled = createChartDocument("flowchart LR");
    expect(shouldUpdateDocumentWindowTitle(null, untitled)).toBe(true);
    expect(shouldUpdateDocumentWindowTitle("제목 없음 — Mermaid Live", untitled)).toBe(false);

    const opened = applyDocumentSnapshot(untitled, snapshot);
    expect(shouldUpdateDocumentWindowTitle("제목 없음 — Mermaid Live", opened)).toBe(true);
    expect(shouldUpdateDocumentWindowTitle("diagram.mmd — Mermaid Live", opened)).toBe(false);

    const dirty = editDocumentSource(opened, "changed");
    expect(shouldUpdateDocumentWindowTitle("diagram.mmd — Mermaid Live", dirty)).toBe(true);
  });
});
