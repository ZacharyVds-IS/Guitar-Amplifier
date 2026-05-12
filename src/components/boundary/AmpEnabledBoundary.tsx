import {ReactNode} from "react";
import {useAmpActiveSync} from "../../hooks/useAmpActiveSync.ts";
import {useAmpStore} from "../../state/AmpConfigStore.tsx";

interface AmpEnabledBoundaryProps {
    children:ReactNode;
    fallback?: ReactNode;
}
export function AmpEnabledBoundary({children, fallback}: AmpEnabledBoundaryProps) {
    const ampActive = useAmpStore((state) => state.is_active);
    useAmpActiveSync();

    return ampActive ? <>{children}</> : <>{fallback}</>;
}