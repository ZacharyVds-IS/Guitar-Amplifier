import {Box, Paper, Stack, Typography} from "@mui/material";

interface FallbackTextProps{
    title:string;
    description:string;
    error?:string;
}
export function FallbackText({title, description, error}:FallbackTextProps) {
    return (
        <Box
            sx={{
                p: 3,
                minHeight: "100vh",
                display: "flex",
                flexDirection: "column",
                gap: 2,
            }}
        >
            <Paper sx={{p: 2, flex: 1, display: "flex", alignItems: "center", justifyContent: "center"}}
                   elevation={2}>
                <Stack sx={{display: "flex", alignItems: "center", justifyContent: "center"}}>
                    <Typography color={"primary"} sx={{fontWeight: "bold", mb: 1}} variant={"h4"}>
                        {title}
                    </Typography>
                    <Typography variant="body2">
                        {description}
                    </Typography>
                    {error && (
                        <Typography variant="caption" color="error" sx={{display: "block", mt: 0.5}}>
                            {error}
                        </Typography>
                    )}
                </Stack>
            </Paper>
        </Box>
    );
}