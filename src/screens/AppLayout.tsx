import {AppBar, Box, Button, Toolbar, Typography} from "@mui/material";
import { Outlet, useNavigate } from "react-router-dom";

export function AppLayout() {
    const navigate = useNavigate();

    return (
        <Box sx={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
            <AppBar
                position="static"
                sx={{
                    height: '50px',
                    justifyContent: 'center',
                    bgcolor: 'background.paper',
                    color: 'text.primary',
                    borderBottom: '1px solid',
                    borderColor: 'divider'
                }}
            >
                <Toolbar variant="dense" sx={{ justifyContent: 'space-between' }}>
                    <Typography variant="h6" sx={{ fontWeight: 'bold' }}>
                        Rust Riff
                    </Typography>
                    <Box>
                        <Button color="inherit" onClick={() => navigate("/")}>Home</Button>
                        <Button color="inherit" onClick={() => navigate("/settings")}>Settings</Button>
                    </Box>
                </Toolbar>
            </AppBar>

            <Box sx={{ flexGrow: 1, overflow: 'auto', bgcolor: 'background.default' }}>
                <Outlet />
            </Box>
        </Box>
    );
}