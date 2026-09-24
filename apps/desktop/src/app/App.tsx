import { EditorPage } from "@/pages/editor";
import { RasterizePage } from "@/pages/rasterize";

function isRasterizeWindow() {
  return new URLSearchParams(window.location.search).get("rasterize") === "1";
}

export function App() {
  return isRasterizeWindow() ? <RasterizePage /> : <EditorPage />;
}
