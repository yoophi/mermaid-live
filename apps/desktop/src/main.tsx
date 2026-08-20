import "@react95/core/GlobalStyle";
import "@react95/core/themes/win95.css";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app/App";
import { AboutWindow } from "./features/show-about";
import "./app/styles/global.css";

const view = new URLSearchParams(window.location.search).get("view");
const isAboutWindow =
  view === "about" || (isTauri() && getCurrentWindow().label === "about-window");

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    {isAboutWindow ? <AboutWindow /> : <App />}
  </StrictMode>,
);
