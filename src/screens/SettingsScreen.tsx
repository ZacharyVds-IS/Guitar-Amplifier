import {Alert, Box, CircularProgress, Typography, useTheme} from "@mui/material";
import {DropdownSelector} from "../components/selection/DropdownSelector.tsx";
import {useAudioDevices} from "../hooks/useAudioDevices.ts";
import {useUpdateAudioDevices} from "../hooks/useUpdateAudioDevices.ts";
import {useState} from "react";

export function SettingsScreen() {
    const theme = useTheme();
    const { inputs, outputs, isLoading, error } = useAudioDevices();
    const { updateInputDevice, updateOutputDevice, error: routingError } = useUpdateAudioDevices();

    const [selectedInput, setSelectedInput] = useState<string>("");
    const [selectedOutput, setSelectedOutput] = useState<string>("");

    const [inputSampleRate, setInputSampleRate] = useState<number | null>(null);
    const [outputSampleRate, setOutputSampleRate] = useState<number | null>(null);

    const inputOptions = inputs.map(d => ({
        label: `${d.name} (${d.sample_rate} Hz)`,
        value: d.id
    }));

    const outputOptions = outputs.map(d => ({
        label: `${d.name} (${d.sample_rate} Hz)`,
        value: d.id
    }));

    async function handleInputChange(id: string) {
        const device = inputs.find(d => d.id === id);
        setSelectedInput(id);
        setInputSampleRate(device?.sample_rate ?? null);
        await updateInputDevice(id);
    }

    async function handleOutputChange(id: string) {
        const device = outputs.find(d => d.id === id);
        setSelectedOutput(id);
        setOutputSampleRate(device?.sample_rate ?? null);
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

            {inputSampleRate &&
                outputSampleRate &&
                inputSampleRate !== outputSampleRate && (
                    <Typography variant="body1">
                        <Box
                            component="span"
                            sx={{ color: theme.palette.primary.main, fontWeight: "bold" }}
                        >
                            Sample rates do not match!
                        </Box>{" "}
                        Output will have a sample rate of:{" "}
                        <Box component="span" sx={{ fontWeight: "bold",color:theme.palette.primary.main }}>
                            {outputSampleRate} Hz
                        </Box>
                    </Typography>
                )}

        </Box>
    );
}

