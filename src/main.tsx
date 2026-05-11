import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import {useAmpStore} from "./state/AmpConfigStore.tsx";
import {listen} from "@tauri-apps/api/event";
import {ChannelDto} from "./domain";

type RustriffWindow = Window & {
    __RUSTRIFF_WINDOW_KIND?: string;
};

const runtimeWindow = window as RustriffWindow;
const isAnalyzerWindow =
    runtimeWindow.__RUSTRIFF_WINDOW_KIND === "analyzer" ||
    window.location.hash.startsWith("#/analyzer");

async function configureListeners() {
    await useAmpStore.getState().init();

    await listen<ChannelDto>("channel-added", (event) => {
        console.log("[event] channel-added", event.payload);
        useAmpStore.getState().addChannelFromBackend(event.payload);
    });
}

if (!isAnalyzerWindow) {
    configureListeners().catch((error) => {
        console.error("Failed to configure backend listeners", error);
    });
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
