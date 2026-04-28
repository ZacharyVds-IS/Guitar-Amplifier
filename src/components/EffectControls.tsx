import {Box, Stack, Typography} from "@mui/material";
import {Knob} from "./selection/Knob.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {FlipSwitch} from "./selection/FlipSwitch.tsx";

export function EffectControls() {

    const activeChannel = useAmpStore((state) =>
        state.channels.find((c) => c.id === state.current_channel)
    );

    const volume = activeChannel?.volume ?? 0;
    const gain = activeChannel?.gain ?? 0;
    const bass = activeChannel?.tone_stack.bass ?? 0;
    const middle = activeChannel?.tone_stack.middle ?? 0;
    const treble = activeChannel?.tone_stack.treble ?? 0;

    const masterVolume = useAmpStore((state) => state.master_volume);
    const isActive = useAmpStore((state) => state.is_active);

    const setVolume = useAmpStore((state) => state.setVolume);
    const setMasterVolume = useAmpStore((state) => state.setMasterVolume);
    const setGain = useAmpStore((state) => state.setGain);
    const setIsActive = useAmpStore((state) => state.setIsActive);

    const setBass = useAmpStore((state) => state.setBass);
    const setMiddle = useAmpStore((state) => state.setMiddle);
    const setTreble = useAmpStore((state) => state.setTreble);


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
            <Stack direction="row" spacing={4}>
                <FlipSwitch label={"On/Off"} value={isActive} onChange={setIsActive}/>
                <Knob
                    label="Volume"
                    value={volume}
                    min={0}
                    max={11}
                    step={1}
                    onChange={setVolume}
                />
                <Knob
                    label="Gain"
                    min={0}
                    max={11}
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
                        <Knob label="Bass" min={0} max={100} value={bass} size={50} onChange={setBass}/>
                        <Knob label="Middle" min={0} max={100} value={middle} size={50} onChange={setMiddle}/>
                        <Knob label="Treble" min={0} max={100} value={treble} size={50} onChange={setTreble}/>
                    </Stack>
                </Box>
                <Knob label={"Master"} min={0} max={11} step={1} value={masterVolume} onChange={setMasterVolume}/>
            </Stack>
        </Box>
    );
}
