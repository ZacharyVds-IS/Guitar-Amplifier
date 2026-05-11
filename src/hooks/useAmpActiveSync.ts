import {listen, type UnlistenFn} from "@tauri-apps/api/event";
import {useEffect} from "react";
import {AMP_ACTIVE_CHANGED_EVENT, useAmpStore} from "../state/AmpConfigStore.tsx";

export function useAmpActiveSync() {
    useEffect(() => {
        let unlisten: UnlistenFn | null = null;
        let disposed = false;

        const sync = async () => {
            await useAmpStore.getState().init();
            if (disposed) {
                return;
            }

            unlisten = await listen<boolean>(AMP_ACTIVE_CHANGED_EVENT, (event) => {
                useAmpStore.setState({is_active: event.payload});
            });
        };

        void sync();

        return () => {
            disposed = true;
            if (unlisten) {
                unlisten();
            }
        };
    }, []);
}

