import {WebviewWindow} from "@tauri-apps/api/webviewWindow";

export {AnalyzerWindow} from "./AnalyzerWindow.tsx";

const ANALYZER_WINDOW_LABEL = "analyzer-view";
const ANALYZER_ROUTE = "/#/analyzer";
export async function openAnalyzerWindow(): Promise<void> {
    const existingWindow = await WebviewWindow.getByLabel(ANALYZER_WINDOW_LABEL);
    if (existingWindow) {
        await existingWindow.setFocus();
        return;
    }

    const analyzerWindow = new WebviewWindow(ANALYZER_WINDOW_LABEL, {
        title: "RustRiff Analyzer",
        url: ANALYZER_ROUTE,
        width: 1024,
        height: 700,
        resizable: true,
        minimizable: true,
        maximizable: true,
        center: true,
    });

    await analyzerWindow.once("tauri://error", (error) => {
        console.error("Failed to create Analyzer window", error);
    });
}