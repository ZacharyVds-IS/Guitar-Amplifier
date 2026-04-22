import {invoke} from "@tauri-apps/api/core";
import {Box, Button} from "@mui/material";
import {EffectChain} from "../components/EffectChain.tsx";
import {EffectControls} from "../components/EffectControls.tsx";

export function MainScreen() {
    async function startLoopback() {
        await invoke("start_loopback");
    }

    return (
        <Box sx={{ p: 4, display: "flex", flexDirection: "column", gap: 2 }}>
            <EffectChain/>
            <EffectControls/>

            <Button
                variant="contained"
                onClick={startLoopback}
            >
                Start Loopback
            </Button>
        </Box>
    );
}
