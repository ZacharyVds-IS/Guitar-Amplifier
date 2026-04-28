import {DropdownSelector} from "./selection/DropdownSelector.tsx";

interface ChannelSelectorProps {
    channels: { label: string; value: number }[];
    currentChannelIndex: number;
    onChannelChange: (index: number) => void;
    onAdd: () => void;
}

export function ChannelSelector({channels, currentChannelIndex, onChannelChange, onAdd}: ChannelSelectorProps) {
    console.log("Channels: ", channels);
    console.log("Current Channel index: ", currentChannelIndex);

    const selectedChannel = channels.find(ch => ch.value === currentChannelIndex);
    console.log("Current Channel: ", selectedChannel);


    if (!selectedChannel) {
        return (
            <DropdownSelector
                label="Channels"
                options={channels}
                selectedValue=""
                onSelectionChange={(index) =>
                    onChannelChange(index as unknown as number)
                }
                onAdd={onAdd}
            />
        );
    }

    return (
        <DropdownSelector
            label="Channels"
            options={channels}
            selectedValue={selectedChannel.label}
            onSelectionChange={(index) =>
                onChannelChange(index as unknown as number)
            }
            onAdd={onAdd}
        />
    );

}