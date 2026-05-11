import {invoke} from "@tauri-apps/api/core";
import {listen, type UnlistenFn} from "@tauri-apps/api/event";
import {useEffect, useState} from "react";

const LIVE_SPECTRUM_EVENT = "live-spectrum";
const ATTACK_ALPHA = 0.18;
const RELEASE_ALPHA = 0.08;
const JITTER_DEADBAND_DB = 0.35;

export type SpectrumSnapshot = {
    sample_rate_hz: number;
    frequencies_hz: number[];
    magnitudes: number[];
    level_db: number;
};

export function useLiveSpectrum() {
    const [spectrum, setSpectrum] = useState<SpectrumSnapshot | null>(null);
    const [loadError, setLoadError] = useState<string | null>(null);

    useEffect(() => {
        let disposed = false;
        let unlisten: UnlistenFn | null = null;

        const bind = async () => {
            try {
                unlisten = await listen<SpectrumSnapshot>(LIVE_SPECTRUM_EVENT, (event) => {
                    if (disposed) {
                        return;
                    }
                    setSpectrum((previous) => blendSpectrum(previous, event.payload));
                    setLoadError(null);
                });

                const initial = await invoke<SpectrumSnapshot>("get_live_spectrum");
                if (!disposed) {
                    setSpectrum((previous) => blendSpectrum(previous, initial));
                    setLoadError(null);
                }

                await invoke("start_live_spectrum_stream");
            } catch (error) {
                if (!disposed) {
                    setLoadError(error instanceof Error ? error.message : "Failed to read spectrum");
                }
            }
        };

        void bind();

        return () => {
            disposed = true;
            if (unlisten) {
                unlisten();
            }
            void invoke("stop_live_spectrum_stream");
        };
    }, []);

    return {spectrum, loadError};
}

function blendSpectrum(previous: SpectrumSnapshot | null, next: SpectrumSnapshot): SpectrumSnapshot {
    if (!previous || previous.magnitudes.length !== next.magnitudes.length) {
        return next;
    }

    return {
        ...next,
        magnitudes: next.magnitudes.map((value, index) => {
            const prev = previous.magnitudes[index] ?? value;
            const delta = value - prev;
            if (Math.abs(delta) < JITTER_DEADBAND_DB) {
                return prev;
            }
            const alpha = delta > 0 ? ATTACK_ALPHA : RELEASE_ALPHA;
            return prev + delta * alpha;
        }),
    };
}


