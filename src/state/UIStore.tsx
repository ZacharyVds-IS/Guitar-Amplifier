import {create} from "zustand/react";

interface UIState {
    developerMode: boolean;
    setDeveloperMode: (show: boolean) => void;
    selectedInputId: string;
    setSelectedInputId: (id: string) => void;
    selectedOutputId: string;
    setSelectedOutputId: (id: string) => void;
}

export const useUIStore = create<UIState>((set) => ({
    developerMode: false,
    setDeveloperMode: (show: boolean) => set({ developerMode: show }),
    selectedInputId: "",
    setSelectedInputId: (id: string) => set({ selectedInputId: id }),
    selectedOutputId: "",
    setSelectedOutputId: (id: string) => set({ selectedOutputId: id }),
}));
