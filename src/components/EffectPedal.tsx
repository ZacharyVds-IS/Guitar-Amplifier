import {Box, Stack, Typography} from "@mui/material";
import chroma from "chroma-js";
import {Knob} from "./selection/Knob.tsx";
import {EffectDto, HcDistortionDto} from "../domain";

interface EffectPedalProps {
    effect: EffectDto;
}

function knobsForEffect(effect: EffectDto): React.ReactNode {
    switch (effect.kind) {
        case "HCDistortion": {
            const data = effect.data as HcDistortionDto;
            return (
                <Knob
                    label="Threshold"
                    value={data.threshold * 100}
                    min={0}
                    max={100}
                    step={0.1}
                    size={40}
                />
            );
        }
        default:
            return null;
    }
}

export function EffectPedal({effect}: EffectPedalProps) {
    const chassisColor = chroma(effect.data.color).hex();

    return (
        <Box
            sx={{
                width: 180,
                height: 280,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                position: 'relative',
                filter: 'drop-shadow(0 6px 12px rgba(0,0,0,0.4))',
            }}
        >
            {/* Top Chassis */}
            <Box
                sx={{
                    width: '100%',
                    height: '60%',
                    background: `linear-gradient(180deg, ${chroma(chassisColor).brighten(0.3)}, ${chassisColor})`,
                    borderRadius: '6px 6px 0 0',
                    border: '1px solid rgba(0,0,0,0.4)',
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    pt: 2,
                    zIndex: 2
                }}
            >
                {/* Status LED */}
                <Box
                    sx={{
                        width: 8,
                        height: 8,
                        borderRadius: '50%',
                        bgcolor: effect.data.is_active ? '#00ff00' : '#ff0000',
                        boxShadow: effect.data.is_active ? '0 0 6px #00ff00' : '0 0 6px #ff0000',
                        mb: 2
                    }}
                />

                {/* Effect-specific knobs */}
                <Stack direction="row" spacing={1} sx={{justifyContent: 'center'}}>
                    {knobsForEffect(effect)}
                </Stack>

                <Typography
                    sx={{
                        mt: 'auto',
                        mb: 2,
                        fontWeight: 900,
                        fontSize: '1.2rem',
                        color: 'rgba(0,0,0,0.7)',
                        textTransform: 'uppercase',
                        fontStyle: 'italic'
                    }}
                >
                    {effect.data.name}
                </Typography>
            </Box>

            {/* Wider Boss-Style Footswitch */}
            <Box
                sx={{
                    width: 'calc(100% + 8px)',
                    height: '40%',
                    bgcolor: '#1a1a1a',
                    borderRadius: '2px 2px 8px 8px',
                    border: '2px solid #000',
                    boxShadow: 'inset 0 2px 4px rgba(255,255,255,0.1)',
                    display: 'flex',
                    justifyContent: 'center',
                    alignItems: 'flex-end',
                    pb: 1,
                    cursor: 'pointer',
                    zIndex: 3,
                    transition: 'transform 0.05s',
                    '&:active': {transform: 'scale(0.98) translateY(2px)'}
                }}
            >
                <Box
                    sx={{
                        width: 12,
                        height: 12,
                        borderRadius: '50%',
                        background: 'radial-gradient(circle, #444, #000)',
                        border: '1px solid #333'
                    }}
                />
            </Box>
        </Box>
    );
}