import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";
import { invoke } from "@tauri-apps/api/core";

invoke<boolean>("main_window_created")
    .then(async (start) => {
        if (!start) {
            await invoke("select_home_dir");
        }
    })
    .catch(e => {
        window.alert(e);
    })

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
