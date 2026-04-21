import { invoke } from "@tauri-apps/api/core";
import {
    Alert,
    Box,
    Button,
    CircularProgress
} from "@mui/material";
import { DropdownSelector } from "../components/selection/DropdownSelector.tsx";
import { useAudioDevices } from "../hooks/useAudioDevices.ts";
import { useState } from "react";
import {useUpdateAudioDevices} from "../hooks/useUpdateAudioDevices.ts";

export function MainScreen() {
    const { inputs, outputs, isLoading, error } = useAudioDevices();
    const { updateInputDevice, updateOutputDevice, isSetting, error: routingError } = useUpdateAudioDevices();

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

    async function handleInputChange(id: string) {
        setSelectedInput(id);
        await updateInputDevice(id);
    }

    async function handleOutputChange(id: string) {
        setSelectedOutput(id);
        await updateOutputDevice(id);
    }

    if (isLoading) return <CircularProgress />;
    if (error) return <Alert severity="error">{error}</Alert>;

    return (
        <Box sx={{ p: 4, display: "flex", flexDirection: "column", gap: 2 }}>
            {(routingError) && <Alert severity="error">{routingError}</Alert>}

            <Button
                variant="contained"
                onClick={startLoopback}
                disabled={!selectedInput || !selectedOutput || isSetting}
            >
                {isSetting ? "Applying..." : "Start Loopback"}
            </Button>

            <DropdownSelector
                title="Input Device"
                label="Select input device"
                options={inputOptions}
                selectedValue={selectedInput}
                onSelectionChange={handleInputChange}
            />

            <DropdownSelector
                title="Output Device"
                label="Select output device"
                options={outputOptions}
                selectedValue={selectedOutput}
                onSelectionChange={handleOutputChange}
            />
            <Box>
                <Typography>Gain</Typography>
                <Slider defaultValue={1.0} max={10} step={0.1} onChange={handleGainChange} valueLabelDisplay="auto"/>
            </Box>
            <Typography variant="h6">{greetMsg}</Typography>
        </Box>
    );
}
