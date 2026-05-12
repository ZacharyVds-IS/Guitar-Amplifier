// @vitest-environment jsdom
import React, {act, useEffect} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";
import {setInputDevice, setOutputDevice} from "../../domain";
import {useUpdateAudioDevices} from "../../hooks/useUpdateAudioDevices";

vi.mock("../../domain", () => ({
    setInputDevice: vi.fn(),
    setOutputDevice: vi.fn(),
}));

type HookValue = ReturnType<typeof useUpdateAudioDevices>;

function Probe({onChange}: {onChange: (value: HookValue) => void}) {
    const value = useUpdateAudioDevices();

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

describe("useUpdateAudioDevices", () => {
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

    it("updates input and output devices successfully", async () => {
        // Arrange
        vi.mocked(setInputDevice).mockResolvedValue(undefined);
        vi.mocked(setOutputDevice).mockResolvedValue(undefined);
        let latest: HookValue | null = null;

        await act(async () => {
            root.render(<Probe onChange={(value) => (latest = value)} />);
            await flush();
        });

        const latestValue = requireLatest(latest);

        // Act
        await act(async () => {
            await latestValue.updateInputDevice("input-1");
            await latestValue.updateOutputDevice("output-1");
            await flush();
            await flush();
        });

        const updatedValue = requireLatest(latest);

        // Assert
        expect(setInputDevice).toHaveBeenCalledWith({deviceId: "input-1"});
        expect(setOutputDevice).toHaveBeenCalledWith({deviceId: "output-1"});
        expect(updatedValue.isSetting).toBe(false);
        expect(updatedValue.error).toBeNull();

        await act(async () => {
            root.unmount();
        });
    });

    it("captures error when updateInputDevice fails", async () => {
        // Arrange
        vi.mocked(setInputDevice).mockRejectedValueOnce(new Error("set input failed"));
        let latest: HookValue | null = null;

        await act(async () => {
            root.render(<Probe onChange={(value) => (latest = value)} />);
            await flush();
        });

        const latestValue = requireLatest(latest);

        // Act
        await act(async () => {
            await latestValue.updateInputDevice("bad-input");
            await flush();
            await flush();
        });

        const updatedValue = requireLatest(latest);

        // Assert
        expect(setInputDevice).toHaveBeenCalledWith({deviceId: "bad-input"});
        expect(updatedValue.isSetting).toBe(false);
        expect(updatedValue.error).toBe("set input failed");

        await act(async () => {
            root.unmount();
        });
    });
});
