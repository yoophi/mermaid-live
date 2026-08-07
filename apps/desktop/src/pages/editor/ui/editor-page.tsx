import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { initialDiagram } from "@/entities/diagram/model/sample";
import { useChartDocument } from "@/features/manage-chart-document";
import { DiagramWorkspace } from "@/widgets/diagram-workspace";

export function EditorPage() {
  const { document, editSource, loadUntitledSource } = useChartDocument(initialDiagram);

  useEffect(() => {
    const sourceFile = new URLSearchParams(window.location.search).get("sourceFile");
    if (!sourceFile) {
      return;
    }

    let cancelled = false;
    invoke<string>("read_diagram_file", { path: sourceFile })
      .then((fileSource) => {
        if (!cancelled) {
          loadUntitledSource(fileSource);
        }
      })
      .catch((error) => {
        console.error("Failed to read diagram file", error);
      });

    return () => {
      cancelled = true;
    };
  }, [loadUntitledSource]);

  return <DiagramWorkspace source={document.source} onSourceChange={editSource} />;
}
