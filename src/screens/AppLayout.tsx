import {AppBar, Box, Button, Toolbar, Typography} from "@mui/material";
import {Outlet, useNavigate} from "react-router-dom";
import {useChannels} from "../hooks/useChannels.ts";
import {ChannelSelector} from "../components/ChannelSelector.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {useEffect} from "react";

export function AppLayout() {
    const navigate = useNavigate();
    const {channels, loading, error} = useChannels();
    const ampStore = useAmpStore();
    const currentChannelName = ampStore.current_channel.name;

    console.log("AppLayout - channels:", channels, "loading:", loading, "currentChannelName:", currentChannelName);

    // Find current channel index based on the amp store's current channel name
    const currentChannelIndex = channels.length > 0
        ? channels.findIndex(ch => ch.name === currentChannelName)
        : 0;

    const channelOptions = channels.map((channel, index) => ({name: channel.name, index}));

    // Initialize amp store on mount
    useEffect(() => {
        console.log("AppLayout mounted, initializing amp store");
        ampStore.init();
    }, []);

    const handleChannelChange = async (index: number) => {
        console.log("Changing channel to index:", index);
        await ampStore.setChannelByIndex(index);
    };

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
                    <Box sx={{ display: 'flex', direction:"row", alignItems: 'center', gap: 2 , width: "25%"}}>
                        {!loading && channels.length > 0 ? (
                            <ChannelSelector
                                channels={channelOptions}
                                currentChannelIndex={currentChannelIndex >= 0 ? currentChannelIndex : 0}
                                onChannelChange={handleChannelChange}
                                onAdd={async () => ampStore.init()}
                            />
                        ) : (
                            <Typography variant="body2" sx={{ color: 'text.secondary' }}>
                                {loading ? "Loading channels..." : error ? "Error loading channels" : "No channels"}
                            </Typography>
                        )}
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