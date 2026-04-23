import {
    Alert,
    Box,
    CircularProgress,
    Typography
} from "@mui/material";
import {DropdownSelector} from "../components/selection/DropdownSelector.tsx";
import {useAudioDevices} from "../hooks/useAudioDevices.ts";
import {useUpdateAudioDevices} from "../hooks/useUpdateAudioDevices.ts";
import {useState} from "react";

export function SettingsScreen() {
    const { inputs, outputs, isLoading, error } = useAudioDevices();
    const { updateInputDevice, updateOutputDevice, error: routingError } = useUpdateAudioDevices();

    const [selectedInput, setSelectedInput] = useState<string>("");
    const [selectedOutput, setSelectedOutput] = useState<string>("");

    const inputOptions = inputs.map(d => ({ label: d.name, value: d.id }));
    const outputOptions = outputs.map(d => ({ label: d.name, value: d.id }));

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
            <Typography variant="h6">Settings</Typography>
            {routingError && <Alert severity="error">{routingError}</Alert>}

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
        </Box>
    );
}
