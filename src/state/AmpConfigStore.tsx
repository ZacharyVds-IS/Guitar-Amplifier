import {AmpConfigDto, getAmpConfig, setGain, setMasterVolume, toggleOnOff} from "../domain";
import {create} from "zustand/react";

interface AmpState extends AmpConfigDto {
    init:() => Promise<void>;
    setGain:(val:number) => void;
    setVolume: (val:number) => void;
    setIsActive:(val:boolean) => void;
}

export const useAmpStore = create<AmpState>((set) => ({
    gain: 0,
    master_volume: 0,
    is_active: false,
    isHydrated: false,

    init: async () => {
        try {
            const config = await getAmpConfig();
            set({
                ...config
            });
            console.log("Store hydrated from Rust:", config);
        } catch (error) {
            console.error("Failed to fetch init state from Rust:", error);
        }
    },

    setGain: (val: number) => {
        set({ gain: val });
        setGain({gain:val});
    },

    setVolume: (val: number) => {
        set({ master_volume: val });
        setMasterVolume({masterVolume:val})
    },

    setIsActive:(val: boolean) => {
        set({is_active: val});
        toggleOnOff({isOn: val});
    }
}));