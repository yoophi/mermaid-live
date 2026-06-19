import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

function defaultDiagramFileName() {
  const now = new Date();
  const pad = (value: number) => String(value).padStart(2, "0");

  return [
    now.getFullYear(),
    pad(now.getMonth() + 1),
    pad(now.getDate()),
    "-",
    pad(now.getHours()),
    pad(now.getMinutes()),
    pad(now.getSeconds()),
    ".mmd",
  ].join("");
}

export function useSaveDiagramRequest(source: string) {
  const sourceRef = useRef(source);

  useEffect(() => {
    sourceRef.current = source;
  }, [source]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen("save-current-diagram", () => {
      invoke("save_diagram_file", {
        source: sourceRef.current,
        defaultFileName: defaultDiagramFileName(),
      }).catch((error) => {
        console.error("Failed to save diagram file", error);
      });
    })
      .then((cleanup) => {
        unlisten = cleanup;
      })
      .catch((error) => {
        console.error("Failed to listen for save request", error);
      });

    return () => {
      unlisten?.();
    };
  }, []);
}
