import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { initialDiagram } from "@/entities/diagram/model/sample";
import { useSaveDiagramRequest } from "@/features/save-diagram";
import { DiagramWorkspace } from "@/widgets/diagram-workspace";

export function EditorPage() {
  const [source, setSource] = useState(initialDiagram);
  useSaveDiagramRequest(source);

  useEffect(() => {
    const sourceFile = new URLSearchParams(window.location.search).get("sourceFile");
    if (!sourceFile) {
      return;
    }

    let cancelled = false;
    invoke<string>("read_diagram_file", { path: sourceFile })
      .then((fileSource) => {
        if (!cancelled) {
          setSource(fileSource);
        }
      })
      .catch((error) => {
        console.error("Failed to read diagram file", error);
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return <DiagramWorkspace source={source} onSourceChange={setSource} />;
}
