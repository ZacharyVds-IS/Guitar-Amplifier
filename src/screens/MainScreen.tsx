import { invoke } from "@tauri-apps/api/core";
import {
    Alert,
    Box,
    Button,
    CircularProgress, Slider, Typography
} from "@mui/material";
import { DropdownSelector } from "../components/selection/DropdownSelector.tsx";
import { useAudioDevices } from "../hooks/useAudioDevices.ts";
import { useState } from "react";
import {useUpdateAudioDevices} from "../hooks/useUpdateAudioDevices.ts";
import {setBass, setGain, setMasterVolume, setMiddle, setTreble} from "../domain";

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

    const handleGainChange = async (_event: Event, value: number | number[]) => {
        const gain = Array.isArray(value) ? value[0] : value;
        await setGain({gain});
    }

    const handleMVChange = async (_event: Event, value: number | number[]) => {
        const masterVolume = Array.isArray(value) ? value[0] : value;
        await setMasterVolume({masterVolume});
    }

    const handleBassChange = async (_event: Event, value: number | number[]) => {
        const bass = Array.isArray(value) ? value[0] : value;
        console.log(`Setting bass to ${bass}`);
        await setBass({bass});
    }

    const handleMiddleChange = async (_event: Event, value: number | number[]) => {
        const middle = Array.isArray(value) ? value[0] : value;
        await setMiddle({middle});
    }

    const handleTrebleChange = async (_event: Event, value: number | number[]) => {
        const treble = Array.isArray(value) ? value[0] : value;
        await setTreble({treble});
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
                <Typography variant="h6">Gain</Typography>
                <Slider aria-label="Gain" defaultValue={1.0} max={10} step={0.1} onChange={handleGainChange} valueLabelDisplay="auto"/>
            </Box>
            <Box>
                <Typography variant="h6">Master Volume</Typography>
                <Slider aria-label="Master Volume" defaultValue={1.0} max={10} step={0.1} onChange={handleMVChange} valueLabelDisplay="auto"/>
            </Box>
            <Box>
                <Typography variant="h6">Tone Stack</Typography>
                <Box sx={{ display: "flex", flexDirection:"row", gap: 4, height: 200, paddingX:5}}>
                    <Box sx={{ display: "flex", flexDirection:"column", gap: 2, height: "100%", alignItems:"center"}}>
                        <Typography>Bass</Typography>
                        <Slider aria-label="Bass" defaultValue={100} orientation="vertical" valueLabelDisplay="auto" onChange={handleBassChange}/>
                    </Box>
                    <Box sx={{ display: "flex", flexDirection:"column", gap: 2, height: "100%", alignItems:"center" }}>
                        <Typography>Middle</Typography>
                        <Slider aria-label="Bass" defaultValue={100} orientation="vertical" valueLabelDisplay="auto" onChange={handleMiddleChange}/>
                    </Box>
                    <Box sx={{ display: "flex", flexDirection:"column", gap: 2, height: "100%", alignItems:"center" }}>
                        <Typography>Treble</Typography>
                        <Slider aria-label="Bass" defaultValue={100} orientation="vertical" valueLabelDisplay="auto" onChange={handleTrebleChange}/>
                    </Box>
                </Box>
            </Box>
        </Box>
    );
}
