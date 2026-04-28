import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import {useAmpStore} from "./state/AmpConfigStore.tsx";
import {listen} from "@tauri-apps/api/event";
import {ChannelDto} from "./domain";

useAmpStore.getState().init();


listen<ChannelDto>("channel-added", (event) => {
    useAmpStore.getState().addChannelFromBackend(event.payload);
})

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
