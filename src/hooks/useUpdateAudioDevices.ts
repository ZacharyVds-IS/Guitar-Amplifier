import {useState} from 'react';
import {setInputDevice, setOutputDevice} from "../domain";

export function useUpdateAudioDevices() {
    const [isSetting, setIsSetting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const toErrorMessage = (err: unknown): string => {
        if (err instanceof Error) {
            return err.message;
        }

        return String(err);
    };

    async function updateInputDevice(deviceId: string) {
        try {
            setIsSetting(true);
            setError(null);
            await setInputDevice({ deviceId });
        } catch (err: unknown) {
            setError(toErrorMessage(err));
        } finally {
            setIsSetting(false);
        }
    }

    async function updateOutputDevice(deviceId: string) {
        try {
            setIsSetting(true);
            setError(null);
            await setOutputDevice({ deviceId });
        } catch (err: unknown) {
            setError(toErrorMessage(err));
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