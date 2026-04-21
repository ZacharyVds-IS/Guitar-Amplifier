import { useState } from 'react';
import {setInputDevice, setOutputDevice} from "../domain";

export function useUpdateAudioDevices() {
    const [isSetting, setIsSetting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    async function updateInputDevice(deviceId: string) {
        try {
            setIsSetting(true);
            setError(null);
            setInputDevice({deviceId});
        } catch (err: any) {
            setError(err.toString());
        } finally {
            setIsSetting(false);
        }
    }

    async function updateOutputDevice(deviceId: string) {
        try {
            setIsSetting(true);
            setError(null);
            setOutputDevice({deviceId});
        } catch (err: any) {
            setError(err.toString());
        } finally {
            setIsSetting(false);
        }
    }

    return {
        updateInputDevice,
        updateOutputDevice,
        isSetting,
        error,
    };
}