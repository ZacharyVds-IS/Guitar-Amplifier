// @vitest-environment jsdom
import React, {act, useEffect} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";
import {getInputDeviceList, getOutputDeviceList} from "../../domain";
import {useAudioDevices} from "../../hooks/useAudioDevices";

vi.mock("../../domain", () => ({
    getInputDeviceList: vi.fn(),
    getOutputDeviceList: vi.fn(),
}));

type HookValue = ReturnType<typeof useAudioDevices>;

function Probe({onChange}: {onChange: (value: HookValue) => void}) {
    const value = useAudioDevices();

    useEffect(() => {
        onChange(value);
    }, [value, onChange]);

    return null;
}

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

function requireLatest(value: HookValue | null): HookValue {
    expect(value).not.toBeNull();
    return value as HookValue;
}

describe("useAudioDevices", () => {
    let container: HTMLDivElement;
    let root: Root;

    beforeAll(() => {
        (globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;
    });

    beforeEach(() => {
        vi.clearAllMocks();
        container = document.createElement("div");
        document.body.appendChild(container);
        root = createRoot(container);
    });

    it("loads input and output devices", async () => {
        // Arrange
        const inputs = [{id: "in-1", name: "Input 1", sample_rate: 48000}];
        const outputs = [{id: "out-1", name: "Output 1", sample_rate: 48000}];
        vi.mocked(getInputDeviceList).mockResolvedValueOnce(inputs as any);
        vi.mocked(getOutputDeviceList).mockResolvedValueOnce(outputs as any);
        let latest: HookValue | null = null;

        // Act
        await act(async () => {
            root.render(<Probe onChange={(value) => (latest = value)} />);
            await flush();
            await flush();
        });

        const latestValue = requireLatest(latest);

        // Assert
        expect(getInputDeviceList).toHaveBeenCalledTimes(1);
        expect(getOutputDeviceList).toHaveBeenCalledTimes(1);
        expect(latestValue.isLoading).toBe(false);
        expect(latestValue.error).toBeNull();
        expect(latestValue.inputs).toEqual(inputs);
        expect(latestValue.outputs).toEqual(outputs);

        await act(async () => {
            root.unmount();
        });
    });

    it("sets error when one device request fails", async () => {
        // Arrange
        vi.mocked(getInputDeviceList).mockRejectedValueOnce(new Error("input failed"));
        vi.mocked(getOutputDeviceList).mockResolvedValueOnce([] as any);
        let latest: HookValue | null = null;

        // Act
        await act(async () => {
            root.render(<Probe onChange={(value) => (latest = value)} />);
            await flush();
            await flush();
        });

        const latestValue = requireLatest(latest);

        // Assert
        expect(latestValue.isLoading).toBe(false);
        expect(latestValue.error).toBe("input failed");

        await act(async () => {
            root.unmount();
        });
    });
});
