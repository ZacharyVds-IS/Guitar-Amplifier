import {createHashRouter} from "react-router-dom";
import {SettingsScreen} from "../../screens/SettingsScreen.tsx";
import {MainScreen} from "../../screens/MainScreen.tsx";
import {AppLayout} from "../../screens/AppLayout.tsx";
import {EqWindow} from "../../windows/EqWindow.tsx";

export const router = createHashRouter([
    {
        path: "/",
        element: <AppLayout />,
        children: [
            {
                index: true,
                element: <MainScreen />,
            },
            {
                path: "settings",
                element: <SettingsScreen />,
            },
        ],
    },
    {
        path: "/eq",
        element: <EqWindow />,
    },
]);