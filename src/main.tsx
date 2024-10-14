import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";
import { invoke } from "@tauri-apps/api/core";

try {
    console.log(await invoke("setup_main_window"));
} catch (e) {
    console.log(e);
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
