import {listen, type UnlistenFn} from "@tauri-apps/api/event";
import {useEffect, useState} from "react";
import {
    getLiveSpectrum,
    getSpectrumContract,
    type SpectrumContractDto,
    type SpectrumSnapshotDto,
    startLiveSpectrumStream,
    stopLiveSpectrumStream,
} from "../domain";

const ATTACK_ALPHA = 0.18;
const RELEASE_ALPHA = 0.08;
const JITTER_DEADBAND_DB = 0.35;

export type LiveSpectrumState = {
    spectrum: SpectrumSnapshotDto | null;
    contract: SpectrumContractDto | null;
    loadError: string | null;
};

export function useLiveSpectrum(): LiveSpectrumState {
    const [spectrum, setSpectrum] = useState<SpectrumSnapshotDto | null>(null);
    const [contract, setContract] = useState<SpectrumContractDto | null>(null);
    const [loadError, setLoadError] = useState<string | null>(null);

    useEffect(() => {
        let disposed = false;
        let unlisten: UnlistenFn | null = null;

        const bind = async () => {
            try {
                const nextContract = await getSpectrumContract();
                if (disposed) {
                    return;
                }
                setContract(nextContract);

                unlisten = await listen<SpectrumSnapshotDto>(nextContract.live_spectrum_event, (event) => {
                    if (disposed) {
                        return;
                    }
                    setSpectrum((previous) => blendSpectrum(previous, event.payload));
                    setLoadError(null);
                });

                const initial = await getLiveSpectrum();
                if (!disposed) {
                    setSpectrum((previous) => blendSpectrum(previous, initial));
                    setLoadError(null);
                }

                await startLiveSpectrumStream();
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
            void stopLiveSpectrumStream();
        };
    }, []);

    return {spectrum, contract, loadError};
}

function blendSpectrum(previous: SpectrumSnapshotDto | null, next: SpectrumSnapshotDto): SpectrumSnapshotDto {
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

