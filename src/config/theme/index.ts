import { createTheme } from "@mui/material";

export const theme = createTheme({
    colorSchemes: {
        light: {
            palette: {
                primary: {
                    main: "#CC6B2C", // The Orange
                    contrastText: "#ffffff",
                },
                secondary: {
                    main: "#1F5E68", // The Teal
                    contrastText: "#ffffff",
                },
                background: {
                    default: "#F2F2F2", // The lightest grey/white
                    paper: "#FFFFFF",   // Pure white
                },
                text: {
                    primary: "#1A1A1A", // Darkest grey for readability
                    secondary: "#2E2E2E",
                },
            },
        },
        dark: {
            palette: {
                primary: {
                    main: "#CC6B2C",
                },
                secondary: {
                    main: "#1F5E68",
                },
                background: {
                    default: "#1A1A1A", // The darkest grey from your image
                    paper: "#2E2E2E",   // The slightly lighter "charcoal" grey
                },
                text: {
                    primary: "#FFFFFF",
                    secondary: "#F2F2F2",
                },
            },
        },
    },
});