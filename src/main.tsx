import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";
import { invoke } from "@tauri-apps/api/core";

try {
    const shouldStart = await invoke<boolean>("main_window_created");
    if (!shouldStart) {
        await invoke("select_home_dir");
    }
} catch(e) {
    console.log(e)
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
