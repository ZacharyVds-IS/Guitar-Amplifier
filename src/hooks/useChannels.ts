import {useEffect, useState} from "react";
import {getAllChannels} from "../domain";

export function useChannels() {
    const [channels, setChannels] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    const fetchChannels = async () => {
        try {
            setLoading(true);
            const data = await getAllChannels();
            setChannels(data);
        } catch (err) {
            console.error("Failed to fetch channels:", err);
            setError(err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchChannels();
    }, [fetchChannels]);

    return {channels, loading, error, refetch: fetchChannels };
}