import {Alert, Box, CircularProgress, Divider, FormControlLabel, Switch, Typography, useTheme} from "@mui/material";
import {DropdownSelector} from "../components/selection/DropdownSelector.tsx";
import {useAudioDevices} from "../hooks/useAudioDevices.ts";
import {useUpdateAudioDevices} from "../hooks/useUpdateAudioDevices.ts";
import {useState} from "react";
import {useUIStore} from "../state/UIStore.tsx";

export function SettingsScreen() {
    const theme = useTheme();
    const { inputs, outputs, isLoading, error } = useAudioDevices();
    const { updateInputDevice, updateOutputDevice, error: routingError } = useUpdateAudioDevices();

    const [selectedInput, setSelectedInput] = useState<string>("");
    const [selectedOutput, setSelectedOutput] = useState<string>("");
    const showLatencyImpacts = useUIStore((state) => state.showLatencyImpacts);
    const setShowLatencyImpacts = useUIStore((state) => state.setShowLatencyImpacts);
    const developerMode = useUIStore((state) => state.developerMode);
    const setDeveloperMode = useUIStore((state) => state.setDeveloperMode);

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
        <Box sx={{ p: 4, display: "flex", flexDirection: "column", height: "100%", gap: 2 }}>
            <Typography variant="h6">Settings</Typography>
            {routingError && <Alert severity="error">{routingError}</Alert>}

            <Box
                sx={{
                    display: "flex",
                    flexDirection: "column",
                    flex: 1,
                    minHeight: 0,
                    overflow: "hidden",
                    backgroundColor: theme.palette.background.paper,
                    borderRadius: 2,
                    boxShadow: 2,
                    p: 3,
                }}
            >
                <Box sx={{ display: "flex", gap: 3, flex: 1, minHeight: 0, overflow: "hidden" }}>
                    {/* Left side - Regular settings (scrollable) */}
                    <Box
                        sx={{
                            flex: "0 0 50%",
                            display: "flex",
                            flexDirection: "column",
                            gap: 2,
                            overflowY: "auto",
                            overflowX: "hidden",
                            pr: 2,
                            "&::-webkit-scrollbar": {
                                width: "8px",
                            },
                            "&::-webkit-scrollbar-track": {
                                background: "transparent",
                            },
                            "&::-webkit-scrollbar-thumb": {
                                background: theme.palette.action.disabled,
                                borderRadius: "4px",
                            },
                        }}
                    >
                        <FormControlLabel
                            control={
                                <Switch
                                    checked={showLatencyImpacts}
                                    onChange={(e) => setShowLatencyImpacts(e.target.checked)}
                                />
                            }
                            label="Show Latency Impacts"
                        />

                        <FormControlLabel
                            control={
                                <Switch
                                    checked={developerMode}
                                    onChange={(e) => setDeveloperMode(e.target.checked)}
                                />
                            }
                            label="Developer Mode"
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

                    {/* Divider */}
                    <Divider orientation="vertical" />

                    {/* Right side - Device settings (non-scrollable) */}
                    <Box
                        sx={{
                            flex: "0 0 50%",
                            display: "flex",
                            flexDirection: "column",
                            gap: 2,
                            pl: 2,
                            overflowX: "hidden",
                        }}
                    >
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
                </Box>
            </Box>
        </Box>
    );
}

