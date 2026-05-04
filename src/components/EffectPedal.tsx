import {Box, Stack, Typography} from "@mui/material";
import chroma from "chroma-js";
import {Knob} from "./selection/Knob.tsx";
import {EffectDto, HcDistortionDto} from "../domain";
import {setHcDistortionLevel, setHcDistortionThreshold, toggleEffect} from "../domain/commands";
import {useState} from "react";

interface EffectPedalProps {
    effect: EffectDto;
    onToggle?: (effectId: number, isActive: boolean) => void;
}

function knobsForEffect(
    effect: EffectDto,
    onParamChange: (name: string, value: number) => void
): React.ReactNode {
    switch (effect.kind) {
        case "HCDistortion": {
            const data = effect.data as HcDistortionDto;
            const THRESHOLD_CLEAN = 1.0;
            const THRESHOLD_HOT   = 0.5;
            const driveKnobValue = (1 - (data.threshold - THRESHOLD_HOT) / (THRESHOLD_CLEAN - THRESHOLD_HOT)) * 100;
            const levelKnobValue = data.level * 100;
            return (
                <>
                    <Knob
                        label="Drive"
                        value={Math.max(0, Math.min(100, driveKnobValue))}
                        min={0}
                        max={100}
                        step={0.5}
                        size={40}
                        valueDisplay="min-max"
                        onChange={(v) => {
                            const threshold = THRESHOLD_CLEAN - (v / 100) * (THRESHOLD_CLEAN - THRESHOLD_HOT);
                            setHcDistortionThreshold({ effectId: data.id, threshold });
                            onParamChange("threshold", threshold);
                        }}
                    />
                    <Knob
                        label="Level"
                        value={Math.max(0, Math.min(100, levelKnobValue))}
                        min={0}
                        max={100}
                        step={0.5}
                        size={40}
                        valueDisplay="min-max"
                        onChange={(v) => {
                            const level = v / 100;
                            setHcDistortionLevel({ effectId: data.id, level });
                            onParamChange("level", level);
                        }}
                    />
                </>
            );
        }
        default:
            return null;
    }
}

export function EffectPedal({effect, onToggle}: EffectPedalProps) {
    const [isActive, setIsActive] = useState(effect.data.is_active);
    const chassisColor = chroma(effect.data.color).hex();

    async function handleFootswitchClick() {
        const newActive = await toggleEffect({ effectId: effect.data.id });
        setIsActive(newActive);
        onToggle?.(effect.data.id, newActive);
    }
    function handleParamChange(_name: string, _value: number) {}

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
                <Box
                    sx={{
                        width: 8,
                        height: 8,
                        borderRadius: '50%',
                        bgcolor: isActive ? '#00ff00' : '#ff0000',
                        boxShadow: isActive ? '0 0 6px #00ff00' : '0 0 6px #ff0000',
                        mb: 2,
                        transition: 'background-color 0.1s, box-shadow 0.1s',
                    }}
                />

                <Stack direction="row" spacing={1} sx={{justifyContent: 'center'}}>
                    {knobsForEffect(effect, handleParamChange)}
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

            <Box
                onClick={handleFootswitchClick}
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