import {ReactNode} from "react";
import {useAmpActiveSync} from "../../hooks/useAmpActiveSync.ts";
import {useAmpStore} from "../../state/AmpConfigStore.tsx";

interface AmpEnabledBoundaryProps {
    children:ReactNode;
    fallback?: ReactNode;
}
export function AmpEnabledBoundary({children, fallback}: AmpEnabledBoundaryProps) {
    const amp_active = useAmpStore((state) => state.is_active);
    useAmpActiveSync();

    return amp_active ? <>{children}</> : <>{fallback}</>;
}