import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import {useAmpStore} from "./state/AmpConfigStore.tsx";

useAmpStore.getState().init();
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
