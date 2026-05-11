import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import {Box, IconButton, Paper, Tooltip, Typography, useTheme} from "@mui/material";
import {LineChart} from "@mui/x-charts/LineChart";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {AmpEnabledBoundary} from "../components/boundary/AmpEnabledBoundary.tsx";
import {FallbackText} from "../components/FallbackText.tsx";
import {useLiveSpectrum} from "../hooks/useLiveSpectrum.ts";

const ANALYZER_WINDOW_LABEL = "analyzer-view";
const ANALYZER_ROUTE = "/#/analyzer";
const MIN_DB = -90;
const MAX_DB = 6;
const MIN_FREQ = 20;
const MAX_FREQ = 20_000;
const FREQ_GRID = [20, 50, 100, 200, 500, 1000, 2000, 5000, 10_000, 20_000];

export async function openAnalyzerWindow(): Promise<void> {
    const existingWindow = await WebviewWindow.getByLabel(ANALYZER_WINDOW_LABEL);
    if (existingWindow) {
        await existingWindow.setFocus();
        return;
    }

    const analyzerWindow = new WebviewWindow(ANALYZER_WINDOW_LABEL, {
        title: "RustRiff Analyzer",
        url: ANALYZER_ROUTE,
        width: 1024,
        height: 700,
        resizable: true,
        minimizable: true,
        maximizable: true,
        center: true,
    });

    await analyzerWindow.once("tauri://error", (error) => {
        console.error("Failed to create Analyzer window", error);
    });
}

export function AnalyzerWindow() {
    const {spectrum, loadError} = useLiveSpectrum();
    const theme = useTheme();

    const chartData =
        spectrum && spectrum.magnitudes.length > 0
            ? spectrum.frequencies_hz.map((freq, index) => ({
                  xPosition: frequencyToUniformGridPosition(freq),
                  magnitude: spectrum.magnitudes[index] ?? MIN_DB,
              }))
            : [];

    return (
        <AmpEnabledBoundary
            fallback={
                <FallbackText
                    title="Analyzer disabled"
                    description="Start audio loopback to restart the analyzer"
                    error={loadError ?? undefined}
                />
            }
        >
            <Box
                sx={{
                    p: 3,
                    minHeight: "100vh",
                    display: "flex",
                    flexDirection: "column",
                    gap: 2,
                    bgcolor: "background.default",
                }}
            >
                <Paper sx={{p: 2}} elevation={2}>
                    <Box sx={{display: "flex", justifyContent: "space-between", alignItems: "flex-start"}}>
                        <Box>
                            <Typography variant="h6">Analyzer Window</Typography>
                            <Typography variant="body2" color="text.secondary">
                                Monitor the current signal passing through the amp.
                            </Typography>
                        </Box>
                    </Box>
                    {loadError && (
                        <Typography variant="caption" color="error" sx={{display: "block", mt: 0.5}}>
                            {loadError}
                        </Typography>
                    )}
                </Paper>

                <Paper
                    sx={{p: 1.5, flex: 1, minHeight: 400, display: "flex", justifyContent: "center", position: "relative"}}
                    elevation={2}
                >
                    <Tooltip
                        arrow
                        placement="left-start"
                        title={
                            <Box sx={{maxWidth: 280}}>
                                <Typography variant="subtitle2" sx={{mb: 0.5}}>
                                    What is dBFS?
                                </Typography>
                                <Typography variant="body2">
                                    dBFS means decibels relative to full scale. 0 dBFS is the maximum digital level before
                                    clipping.
                                </Typography>
                                <Typography variant="body2" sx={{mt: 0.75}}>
                                    In this graph, lower values (for example -24 dBFS) are quieter; values close to 0 dBFS
                                    are louder and near clipping.
                                </Typography>
                            </Box>
                        }
                    >
                        <IconButton
                            size="small"
                            aria-label="Explain dBFS"
                            sx={{
                                position: "absolute",
                                top: 8,
                                right: 8,
                                zIndex: 1,
                                bgcolor: "background.paper",
                                border: 1,
                                borderColor: "divider",
                                "&:hover": {bgcolor: "action.hover"},
                            }}
                        >
                            <InfoOutlinedIcon fontSize="small" />
                        </IconButton>
                    </Tooltip>
                    <LineChart
                        dataset={chartData}
                        skipAnimation
                        xAxis={[
                            {
                                dataKey: "xPosition",
                                scaleType: "linear",
                                min: 0,
                                max: FREQ_GRID.length - 1,
                                label: "Frequency",
                                tickInterval: FREQ_GRID.map((_, index) => index),
                                valueFormatter: (value: number) => formatFreqLabel(frequencyForGridPosition(value)),
                            },
                        ]}
                        yAxis={[
                            {
                                min: MIN_DB,
                                max: MAX_DB,
                                label: "Level (dBFS)",
                            },
                        ]}
                        series={[
                            {
                                dataKey: "magnitude",
                                label: "Magnitude",
                                color: `${theme.palette.primary.main}`,
                                showMark: false,
                                curve: "linear",
                            },
                        ]}
                        width={900}
                        height={500}
                        margin={{top: 10, bottom: 40, left: 60, right: 10}}
                        slots={{
                            legend: () => null,
                        }}
                    />
                </Paper>
            </Box>
        </AmpEnabledBoundary>
    );
}


function frequencyToUniformGridPosition(frequencyHz: number): number {
    const clamped = Math.min(MAX_FREQ, Math.max(MIN_FREQ, frequencyHz));
    const logFreq = Math.log10(clamped);
    const gridCells = FREQ_GRID.length - 1;

    for (let index = 0; index < gridCells; index++) {
        const cellStart = Math.log10(FREQ_GRID[index]);
        const cellEnd = Math.log10(FREQ_GRID[index + 1]);
        if (logFreq >= cellStart && logFreq <= cellEnd) {
            const ratio = (logFreq - cellStart) / (cellEnd - cellStart);
            return index + ratio;
        }
    }

    return gridCells;
}

function frequencyForGridPosition(position: number): number {
    const gridCells = FREQ_GRID.length - 1;
    const clampedPosition = Math.min(gridCells, Math.max(0, position));
    const index = Math.min(gridCells - 1, Math.floor(clampedPosition));
    const ratio = clampedPosition - index;
    const start = Math.log10(FREQ_GRID[index]);
    const end = Math.log10(FREQ_GRID[index + 1]);
    return 10 ** (start + (end - start) * ratio);
}

function formatFreqLabel(frequency: number): string {
    if (frequency >= 1000) {
        const value = frequency / 1000;
        return `${value >= 10 ? value.toFixed(0) : value.toFixed(1)}k`;
    }
    return `${Math.round(frequency)}`;
}