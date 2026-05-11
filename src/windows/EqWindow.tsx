import {Box, Paper, Typography} from "@mui/material";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";

const EQ_WINDOW_LABEL = "eq-view";

export async function openEqWindow(): Promise<void> {
    const existingWindow = await WebviewWindow.getByLabel(EQ_WINDOW_LABEL);
    if (existingWindow) {
        await existingWindow.setFocus();
        return;
    }

    const eqWindow = new WebviewWindow(EQ_WINDOW_LABEL, {
        title: "RustRiff EQ",
        url: "/#/eq",
        width: 980,
        height: 640,
        resizable: true,
        minimizable: true,
        maximizable: true,
        center: true,
    });

    eqWindow.once("tauri://error", (error) => {
        console.error("Failed to create EQ window", error);
    });
}

export function EqWindow() {
    return (
        <Box
            sx={{
                p: 3,
                minHeight: "100vh",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                bgcolor: "background.default",
            }}
        >
            <Paper sx={{p: 4, width: "min(800px, 100%)"}} elevation={3}>
                <Typography variant="h5" gutterBottom>
                    EQ View
                </Typography>
                <Typography variant="body1" color="text.secondary">
                    This is a dedicated screen rendered in the secondary window.
                    Add your EQ controls and visualizer components here.
                </Typography>
            </Paper>
        </Box>
    );
}

