import { Box, Typography, useTheme, ButtonBase } from "@mui/material";

interface FlipSwitchProps {
    label: string;
    value: boolean;
    onChange?: (newValue: boolean) => void;
    size?: number;
}

export function FlipSwitch({ label, value, onChange, size = 50 }: FlipSwitchProps) {
    const theme = useTheme();

    const plateHeight = size * 1.3;
    const plateWidth = size;

    const handleToggle = () => {
        if (onChange) onChange(!value);
    };

    return (
        <Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', width: plateWidth + 20, userSelect: 'none' }}>
            <Typography variant="caption" sx={{ color: theme.palette.text.primary, mb: 1, fontWeight: 600, fontSize: '0.65rem', textTransform: 'uppercase' }}>
                {label}
            </Typography>

            <Box sx={{ width: plateWidth, height: plateHeight, bgcolor: 'grey.300', border: '3px solid', borderColor: theme.palette.background.default, borderRadius: 2, position: 'relative', boxShadow: theme.shadows[4], display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', overflow: 'hidden' }}>
                <Box sx={{ position: 'absolute', width: '10px', height: '75%', bgcolor: 'rgba(0,0,0,0.15)', borderRadius: 1, boxShadow: 'inset 0 1px 3px rgba(0,0,0,0.3)' }} />

                <ButtonBase onClick={handleToggle} disableRipple sx={{ width: '100%', height: '100%', zIndex: 1, cursor: 'pointer' }}>
                    <Box
                        sx={{
                            width: '26px',
                            height: '18px',
                            borderRadius: '3px',
                            border: '2px solid',
                            borderColor: 'common.black',
                            bgcolor: value
                                ? theme.palette.primary.main
                                : theme.palette.secondary.main,
                            transform: `translateY(${value ? '-18px' : '18px'})`,

                            boxShadow: '0 3px 6px rgba(0,0,0,0.4)',
                            transition: theme.transitions.create(['transform', 'background-color'], {
                                duration: 150,
                                easing: theme.transitions.easing.easeInOut,
                            }),

                            '&::after': {
                                content: '""',
                                position: 'absolute',
                                top: '50%',
                                left: '50%',
                                transform: 'translate(-50%, -50%)',
                                width: '50%',
                                height: '2px',
                                bgcolor: 'rgba(0,0,0,0.2)',
                                boxShadow: '0 4px 0 rgba(0,0,0,0.2), 0 -4px 0 rgba(0,0,0,0.2)'
                            }
                        }}
                    />
                </ButtonBase>
            </Box>

            <Typography sx={{ fontSize: '0.6rem', mt: 0.8, color: theme.palette.text.primary, fontFamily: 'monospace', fontWeight: 'bold' }}>
                {value ? "On" : "Off"}
            </Typography>
        </Box>
    );
}