import { useState } from "react";
import { initialDiagram } from "@/entities/diagram/model/sample";
import { DiagramCodeEditor } from "@/features/edit-diagram/ui/diagram-code-editor";
import { MermaidPreview } from "@/features/preview-diagram/ui/mermaid-preview";
import { Badge } from "@/shared/ui/badge";

export function DiagramWorkspace() {
  const [source, setSource] = useState(initialDiagram);

  return (
    <main className="grid h-screen min-h-0 grid-cols-[minmax(280px,1fr)_minmax(0,2fr)] overflow-hidden">
      <section className="flex min-h-0 flex-col border-r bg-card/88">
        <header className="flex h-12 shrink-0 items-center justify-between border-b px-4">
          <div className="text-sm font-semibold">A. Source</div>
          <Badge>CodeMirror</Badge>
        </header>
        <div className="min-h-0 flex-1">
          <DiagramCodeEditor value={source} onChange={setSource} />
        </div>
      </section>

      <section className="flex min-h-0 flex-col bg-background/82">
        <header className="flex h-12 shrink-0 items-center justify-between border-b px-4">
          <div className="text-sm font-semibold">B. Preview</div>
          <Badge>Mermaid</Badge>
        </header>
        <div className="min-h-0 flex-1">
          <MermaidPreview source={source} />
        </div>
      </section>
    </main>
  );
}
