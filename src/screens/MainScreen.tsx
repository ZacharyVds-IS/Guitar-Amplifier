import {Box} from "@mui/material";
import {EffectChain} from "../components/EffectChain.tsx";
import {EffectControls} from "../components/EffectControls.tsx";

export function MainScreen() {
    return (
        <Box sx={{ p: 4, display: "flex", flexDirection: "column", gap: 2 }}>
            <EffectChain/>
            <EffectControls/>
        </Box>
    );
}
