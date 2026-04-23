import { Box, Stack, Typography } from "@mui/material";
import { Knob } from "./selection/Knob.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {FlipSwitch} from "./selection/FlipSwitch.tsx";

export function EffectControls() {
    const volume = useAmpStore((state) => state.master_volume);
    const gain = useAmpStore((state) => state.gain);
    const isActive = useAmpStore((state) => state.is_active);

    const setVolume = useAmpStore((state) => state.setVolume);
    const setGain = useAmpStore((state) => state.setGain);
    const setIsActive = useAmpStore((state) => state.setIsActive);

    const setBass = useAmpStore((state) => state.setBass);
    const setMiddle= useAmpStore((state) => state.setMiddle);
    const setTreble= useAmpStore((state) => state.setTreble);

    return (
        <Box
            sx={{
                p: 4,
                bgcolor: 'background.paper',
                borderRadius: 4,
                display: 'inline-block',
                border: '1px solid',
                borderColor: 'divider',
                boxShadow: 8
            }}
        >
            <Stack direction="row" spacing={4} >
                <FlipSwitch label={"On/Off"} value={isActive} onChange={setIsActive}/>
                <Knob
                    label="Volume"
                    value={volume}
                    min={0}
                    max={10}
                    step={1}
                    onChange={setVolume}
                />
                <Knob
                    label="Gain"
                    min={0}
                    max={10}
                    step={0.1}
                    value={gain}
                    onChange={setGain}
                />
                <Box
                    sx={{
                        border: '1px solid',
                        borderColor: 'divider',
                        p: 2,
                        borderRadius: 2,
                        position: 'relative'
                    }}
                >
                    <Typography
                        sx={{
                            position: 'absolute',
                            top: -10,
                            left: 10,
                            bgcolor: 'background.paper',
                            px: 1,
                            fontSize: '0.7rem',
                            fontWeight: 'bold',
                            color: 'text.secondary',
                            textTransform: 'uppercase',
                            letterSpacing: '0.05rem'
                        }}
                    >
                        Tone stack
                    </Typography>

                    <Stack direction="row" spacing={2}>
                        <Knob label="Bass" min={0} max={100} value={100} size={50} onChange={setBass}/>
                        <Knob label="Middle" min={0} max={100} value={100} size={50} onChange={setMiddle}/>
                        <Knob label="Treble" min={0} max={100} value={100} size={50} onChange={setTreble}/>
                    </Stack>
                </Box>
            </Stack>
        </Box>
    );
}