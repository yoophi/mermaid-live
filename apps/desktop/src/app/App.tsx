import { EditorPage } from "@/pages/editor";
import { closeApplicationWindow, startApplicationWindowDrag } from "@/shared/lib/tauri-window";
import { Window95 } from "@/shared/ui/window-95";

export function App() {
  return (
    <div className="desktop-stage">
      <Window95
        onClose={() => void closeApplicationWindow()}
        onTitleBarMouseDown={(event) => void startApplicationWindowDrag(event)}
        title="Mermaid Live"
      >
        <EditorPage />
      </Window95>
    </div>
  );
}
