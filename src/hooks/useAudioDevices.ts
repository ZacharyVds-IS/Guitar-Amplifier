import { useState, useEffect, useCallback } from 'react';
import {AudioDeviceDto, getInputDeviceList, getOutputDeviceList} from "../domain";

export const useAudioDevices = () => {
    const [inputs, setInputs] = useState<AudioDeviceDto[]>([]);
    const [outputs, setOutputs] = useState<AudioDeviceDto[]>([]);
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [error, setError] = useState<string | null>(null);

    const fetchDevices = useCallback(async () => {
        setIsLoading(true);
        setError(null);
        try {
            const [inputList, outputList] = await Promise.all([
                getInputDeviceList(),
                getOutputDeviceList()
            ]);

            setInputs(inputList);
            setOutputs(outputList);
        } catch (err) {
            setError(err instanceof Error ? err.message : String(err));
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        fetchDevices();
    }, [fetchDevices]);

    return { inputs, outputs, isLoading, error, refresh: fetchDevices };
};