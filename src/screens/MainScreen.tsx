import {Box} from "@mui/material";
import {EffectChain} from "../components/EffectChain.tsx";
import {DefaultAmpControls} from "../components/DefaultAmpControls.tsx";
import {EffectPedal} from "../components/EffectPedal.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {useState} from "react";
import {EffectDto} from "../domain";

export function MainScreen() {
    const activeChannel = useAmpStore((state) =>
        state.channels.find((c) => c.id === state.current_channel)
    );

    const [selection, setSelection] = useState<EffectDto | "amp">("amp");

    return (
        <Box
            sx={{
                p: 4,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "start",
                minHeight: "100vh",
                gap: 4
            }}
        >
            {activeChannel &&
                <EffectChain
                    effects={activeChannel.effect_chain}
                    selected={selection}
                    onSelectionChange={setSelection}
                />
            }

            {selection === "amp"
                ? <DefaultAmpControls/>
                : <EffectPedal effect={selection}/>
            }
        </Box>
    );
}