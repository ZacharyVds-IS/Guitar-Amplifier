import { invoke } from "@tauri-apps/api/core";
import {
    Alert,
    Box,
    Button, CircularProgress
} from "@mui/material";
import {DropdownSelector} from "../components/selection/DropdownSelector.tsx";
import {useAudioDevices} from "../hooks/useAudioDevices.ts";
import {useState} from "react";

export function MainScreen() {
    const { inputs, outputs, isLoading, error } = useAudioDevices();
    const [selectedInput, setSelectedInput] = useState<string>("");
    const [selectedOutput, setSelectedOutput] = useState<string>("");
    const inputOptions = inputs.map(d => ({ label: d.name, value: d.id }));
    const outputOptions = outputs.map(d => ({ label: d.name, value: d.id }));

    async function startLoopback() {
        await invoke("start_loopback", {
            inputId: selectedInput,
            outputId: selectedOutput
        });
    }

    if (isLoading) return <CircularProgress />;
    if (error) return <Alert severity="error">{error}</Alert>;

    return (
        <Box sx={{ p: 4, display: 'flex', flexDirection: 'column', gap: 2 }}>
            <Button
                variant="contained"
                onClick={startLoopback}
                disabled={!selectedInput || !selectedOutput}
            >
                Start Loopback
            </Button>

            <DropdownSelector
                title={"Input Device"}
                label={"Select input device"}
                options={inputOptions}
                selectedValue={selectedInput}
                onSelectionChange={(val) => setSelectedInput(val)}
            />

            <DropdownSelector
                title={"Output Device"}
                label={"Select output device"}
                options={outputOptions}
                selectedValue={selectedOutput}
                onSelectionChange={(val) => setSelectedOutput(val)}
            />
        </Box>
    );
}