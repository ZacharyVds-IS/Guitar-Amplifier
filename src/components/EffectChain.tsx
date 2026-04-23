import { Box, Stack, Typography,Paper } from "@mui/material";

const effects = [
    {
        id: 1,
        type: 'Amp',
        icon: (
            <Box
                sx={{
                    width: 60,
                    height: 60,
                    bgcolor: 'background.paper',
                    border: '1px solid',
                    borderColor: 'text.secondary',
                    borderRadius: 2
                }}
            />
        )
    },
    {
        id: 2,
        type: 'Distortion',
        icon: (
            <Box
                sx={{
                    width: 40,
                    height: 70,
                    bgcolor: '#1E1E1E',
                    borderRadius: 4,
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: 'center',
                    p: 1,
                    boxShadow: 2
                }}
            >
                <Typography variant="caption" sx={{ color: '#F2F2F2', fontSize: '0.6rem', mb: 0.5 }}>
                    Distortion
                </Typography>
                <Stack direction="row" spacing={1}>
                    {/* Two knobs on top */}
                    <Box sx={{ width: 10, height: 10, bgcolor: '#FFFFFF', borderRadius: '50%' }} />
                    <Box sx={{ width: 10, height: 10, bgcolor: '#FFFFFF', borderRadius: '50%' }} />
                </Stack>
                <Stack direction="row" spacing={1} sx={{ color: '#999', fontSize: '0.5rem' }}>
                    <Typography variant="caption" sx={{ fontSize: 'inherit' }}>Tone</Typography>
                    <Typography variant="caption" sx={{ fontSize: 'inherit' }}>Level</Typography>
                </Stack>
                <Box sx={{ width: 15, height: 15, bgcolor: '#FFFFFF', borderRadius: '50%', boxShadow: 1 }} />
            </Box>
        )
    }
];

export function EffectChain() {
    return (
        <Box
            component="section"
            sx={{
                width: '100%',
                bgcolor: 'background.paper',
                borderRadius: 4,
                p: 2,
                position: 'relative'
            }}
        >

            <Box sx={{ display: 'flex', justifyContent: 'flex-end', mb: 4 }}>
                <Paper
                    sx={{
                        bgcolor: 'background.paper',
                        color: 'primary.main',
                        borderRadius: 50,
                        textTransform: 'none',
                        fontSize: '0.875rem',
                        fontWeight: 500,
                        p: 1.2,
                        px: 3,
                        border: '1px solid',
                        borderColor: 'divider',
                        '&:hover': {
                            bgcolor: '#fdfdfd',
                            cursor: 'pointer'
                        }
                    }}
                >
                    Edit order
                </Paper>
            </Box>

            <Box
                sx={{
                    position: 'absolute',
                    left: 0,
                    right: 0,
                    top: '60%',
                    transform: 'translateY(-50%)',
                    height: '6px',
                    bgcolor: 'secondary.main',
                    zIndex: 1
                }}
            />
            <Stack
                direction="row"
                spacing={6}
                sx={{
                    width: '100%',
                    position: 'relative',
                    zIndex: 2
                }}
            >
                {effects.map((item) => (
                    <Box key={item.id} sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
                        <Box sx={{ display: 'flex', alignItems: 'center', height: 75 }}>
                            {item.icon}
                        </Box>
                        <Typography
                            variant="caption"
                            sx={{
                                mt: 1,

                                color: 'text.primary',
                                fontWeight: 500,
                                fontSize: '0.75rem'
                            }}
                        >
                            {item.type}
                        </Typography>
                    </Box>
                ))}
            </Stack>
        </Box>
    );
}