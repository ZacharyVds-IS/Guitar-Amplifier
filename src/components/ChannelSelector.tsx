import {Box} from "@mui/material";
import {DropdownSelector} from "./selection/DropdownSelector.tsx";

interface ChannelSelectorProps {
    channels: { name: string; index: number }[];
    currentChannelIndex: number;
    onChannelChange: (index: number) => void;
    onAdd: () => void;
}

export function ChannelSelector({channels, currentChannelIndex, onChannelChange, onAdd}: ChannelSelectorProps) {
    return (
        <Box sx={{display: 'flex', direction: "row", alignItems: 'center', gap: 2, width: "100%"}}>
            <DropdownSelector
                label={"Channels"} options={channels} selectedValue={channels[currentChannelIndex]}
                onSelectionChange={onChannelChange()} onAdd={onAdd}/>
        </Box>
    )
}