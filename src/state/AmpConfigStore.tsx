import {
    AmpConfigDto,
    getAmpConfig,
    setBass,
    setGain,
    setMasterVolume,
    setMiddle,
    setTreble,
    setVolume,
    toggleOnOff
} from "../domain";
import {create} from "zustand/react";

interface AmpState extends AmpConfigDto {
    init: () => Promise<void>;
    setGain: (val: number) => void;
    setVolume: (val: number) => void;
    setMasterVolume: (val: number) => void;
    setIsActive: (val: boolean) => void;
    setBass: (val: number) => void;
    setMiddle: (val: number) => void;
    setTreble: (val: number) => void;
}

export const useAmpStore = create<AmpState>((set) => ({
    master_volume: 1,
    is_active: false,
    current_channel: {
        name: "",
        gain: 1.0,
        tone_stack: {
            bass: 1.0,
            middle: 1.0,
            treble: 1.0,
        },
        volume: 1,
    },

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

    setMasterVolume: (val: number) => {
        set({master_volume: val});
        setMasterVolume({masterVolume: val})
    },

    setIsActive: (val: boolean) => {
        set({is_active: val});
        toggleOnOff({isOn: val});
    },

    setGain: (val: number) => {
        set((state) => ({
            current_channel: {
                ...state.current_channel,
                gain: val
            }
        }));
        setGain({gain: val});
    },

    setVolume: (val: number) => {
        set((state)=>({
            current_channel: {
                ...state.current_channel,
                volume: val,
            }
        }));
        setVolume({volume: val})
    },

    setBass: (val: number) => {
        set((state) => ({
            current_channel: {
                ...state.current_channel,
                tone_stack: {
                    ...state.tone_stack,
                    bass: val,
                },
            }
        }));
        setBass({bass: val})
    },

    setMiddle: (val: number) => {
        set((state) => ({
            current_channel: {
                ...state.current_channel,
                tone_stack: {
                    ...state.tone_stack,
                    middle: val,
                },
            }
        }));
        setMiddle({middle: val})
    },

    setTreble: (val: number) => {
        set((state) => ({
            current_channel: {
                ...state.current_channel,
                tone_stack: {
                    ...state.tone_stack,
                    treble: val,
                },
            }
        }));
        setTreble({treble: val})
    },
}));