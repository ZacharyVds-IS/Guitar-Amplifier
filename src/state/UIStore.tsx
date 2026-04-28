import {create} from "zustand/react";

interface UIState {
    developerMode: boolean;
    setDeveloperMode: (show: boolean) => void;
}

export const useUIStore = create<UIState>((set) => ({
    developerMode: false,
    setDeveloperMode: (show: boolean) => {
        set({ developerMode: show });
    },
}));

