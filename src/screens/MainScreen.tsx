import {useState} from "react";
import {invoke} from "@tauri-apps/api/core";
import {
    Box,
    Button, Slider,
    TextField,
    Typography
} from "@mui/material";

export function MainScreen() {
    const [greetMsg, setGreetMsg] = useState("");
    const [name, setName] = useState("");

    async function greet() {
        setGreetMsg(await invoke("greet", {name}));
    }

    async function startLoopback() {
        await invoke("start_loopback");
    }

    const handleGainChange = async (_event: Event, value: number) => {
        await invoke("set_gain", {value});
    }

    const handleMVChange =async (_event: Event, value: number,)=> {
        await invoke("set_master_volume", {value});
    }

    return (
        <Box sx={{p: 4}}>
            <Typography variant="h4" gutterBottom>
                Welcome to Tauri + React
            </Typography>

            <Typography sx={{mb: 3}}>
                Click on the Tauri, Vite, and React logos to learn more.
            </Typography>

            <Box
                component="form"
                onSubmit={(e) => {
                    e.preventDefault();
                    greet();
                }}
                sx={{display: "flex", gap: 2, alignItems: "center", mb: 3}}
            >
                <TextField
                    id="greet-input"
                    label="Enter a name..."
                    variant="outlined"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                />

                <Button variant="contained" type="submit">
                    Greet
                </Button>
            </Box>
            <Button variant="contained" onClick={startLoopback}>
                Start Loopback
            </Button>
            <Box>
                <Typography>Gain</Typography>
                <Slider defaultValue={1.0} max={10} step={0.1} onChange={handleMVChange} valueLabelDisplay="auto"/>
            </Box>
            <Box>
                <Typography>Master Volume</Typography>
                <Slider defaultValue={1.0} max={10} step={0.1} onChange={handleGainChange} valueLabelDisplay="auto"/>
            </Box>
            <Typography variant="h6">{greetMsg}</Typography>
        </Box>
    );
}
