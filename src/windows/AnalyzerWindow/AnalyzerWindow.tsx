import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import {Box, IconButton, Paper, Tooltip, Typography, useTheme} from "@mui/material";
import {LineChart} from "@mui/x-charts/LineChart";
import {AmpEnabledBoundary} from "../../components/boundary/AmpEnabledBoundary.tsx";
import {FallbackText} from "../../components/FallbackText.tsx";
import {useLiveSpectrum} from "../../hooks/useLiveSpectrum.ts";

const DEFAULT_MIN_DB = -90;
const DEFAULT_MAX_DB = 6;
const DEFAULT_MIN_FREQ = 20;
const DEFAULT_MAX_FREQ = 20_000;
const FREQ_GRID = [20, 50, 100, 200, 500, 1000, 2000, 5000, 10_000, 20_000];

export function AnalyzerWindow() {
    const {spectrum, contract, loadError} = useLiveSpectrum();
    const theme = useTheme();

    const minDb = contract?.min_db ?? DEFAULT_MIN_DB;
    const maxDb = contract?.max_db ?? DEFAULT_MAX_DB;
    const minFrequencyHz = contract?.min_frequency_hz ?? DEFAULT_MIN_FREQ;
    const maxFrequencyHz = contract?.max_frequency_hz ?? DEFAULT_MAX_FREQ;

    const chartData =
        spectrum && spectrum.magnitudes.length > 0
            ? spectrum.frequencies_hz.map((freq, index) => ({
                  xPosition: frequencyToUniformGridPosition(freq, minFrequencyHz, maxFrequencyHz),
                  magnitude: spectrum.magnitudes[index] ?? minDb,
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
                                min: minDb,
                                max: maxDb,
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


function frequencyToUniformGridPosition(frequencyHz: number, minFrequencyHz: number, maxFrequencyHz: number): number {
    const clamped = Math.min(maxFrequencyHz, Math.max(minFrequencyHz, frequencyHz));
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