// @vitest-environment jsdom
import React, {act, useEffect} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";
import {getAllChannels} from "../../domain";
import {useChannels} from "../../hooks/useChannels";

vi.mock("../../domain", () => ({
    getAllChannels: vi.fn(),
}));

type HookValue = ReturnType<typeof useChannels>;

function Probe({onChange}: {onChange: (value: HookValue) => void}) {
    const value = useChannels();

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

describe("useChannels", () => {
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

    it("loads channels successfully", async () => {
        // Arrange
        const data = [{id: 1, name: "Lead", gain: 1, tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5}, volume: 1, effect_chain: []}];
        vi.mocked(getAllChannels).mockResolvedValueOnce(data as any);
        let latest: HookValue | null = null;

        // Act
        await act(async () => {
            root.render(<Probe onChange={(value) => (latest = value)} />);
            await flush();
            await flush();
        });

        const latestValue = requireLatest(latest);

        // Assert
        expect(getAllChannels).toHaveBeenCalledTimes(1);
        expect(latestValue.loading).toBe(false);
        expect(latestValue.error).toBeNull();
        expect(latestValue.channels).toEqual(data);

        await act(async () => {
            root.unmount();
        });
    });

    it("sets error when backend request fails", async () => {
        // Arrange
        vi.mocked(getAllChannels).mockRejectedValueOnce(new Error("channels failed"));
        let latest: HookValue | null = null;

        // Act
        await act(async () => {
            root.render(<Probe onChange={(value) => (latest = value)} />);
            await flush();
            await flush();
        });

        const latestValue = requireLatest(latest);

        // Assert
        expect(latestValue.loading).toBe(false);
        expect(latestValue.channels).toEqual([]);
        expect(latestValue.error).toBe("channels failed");

        await act(async () => {
            root.unmount();
        });
    });
});
