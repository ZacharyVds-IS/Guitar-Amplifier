import {beforeEach, describe, expect, it, vi} from "vitest";
import * as domain from "../../domain";
import {useAmpStore} from "../../state/AmpConfigStore";

vi.mock("../../domain", () => ({
    addChannel: vi.fn(),
    addEffect: vi.fn(),
    applyEffectOrderChange: vi.fn(),
    getAmpConfig: vi.fn(),
    removeChannel: vi.fn(),
    removeEffect: vi.fn(),
    setBass: vi.fn(),
    setChannelId: vi.fn(),
    setGain: vi.fn(),
    setMasterVolume: vi.fn(),
    setMiddle: vi.fn(),
    setTreble: vi.fn(),
    setVolume: vi.fn(),
    toggleOnOff: vi.fn().mockResolvedValue(undefined),
}));

const distortionEffect = {
    kind: "HCDistortion",
    data: {
        id: 10,
        name: "Drive",
        is_active: true,
        threshold: 0.4,
        level: 0.5,
    },
} as const;

const delayEffect = {
    kind: "Delay",
    data: {
        id: 11,
        name: "Delay",
        is_active: true,
        delay_time: 220,
        level: 0.4,
    },
} as const;

function resetStore() {
    useAmpStore.setState({
        master_volume: 1,
        is_active: false,
        current_channel: 0,
        chain_snapshot: null,
        channels: [
            {
                id: 0,
                name: "Clean",
                gain: 1,
                tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5},
                volume: 1,
                effect_chain: [distortionEffect, delayEffect] as any,
            },
            {
                id: 1,
                name: "Lead",
                gain: 1.2,
                tone_stack: {bass: 0.6, middle: 0.6, treble: 0.6},
                volume: 0.8,
                effect_chain: [],
            },
        ],
    });
}

describe("AmpConfigStore", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        resetStore();
    });

    describe("success_path", () => {
        it("init hydrates store from backend", async () => {
            // Arrange
            const backendConfig = {
                master_volume: 0.7,
                is_active: true,
                current_channel: 1,
                channels: [{id: 1, name: "Lead", gain: 1.1, tone_stack: {bass: 0.4, middle: 0.6, treble: 0.8}, volume: 0.9, effect_chain: []}],
            };
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce(backendConfig as any);

            // Act
            await useAmpStore.getState().init();

            // Assert
            expect(useAmpStore.getState().master_volume).toBe(0.7);
            expect(useAmpStore.getState().is_active).toBe(true);
            expect(useAmpStore.getState().current_channel).toBe(1);
        });

        it("setChannelById updates current channel and refreshes config", async () => {
            // Arrange
            const backendConfig = {
                master_volume: 0.9,
                is_active: true,
                current_channel: 1,
                channels: [{id: 1, name: "Lead", gain: 1.2, tone_stack: {bass: 0.7, middle: 0.6, treble: 0.8}, volume: 0.75, effect_chain: []}],
            };
            vi.mocked(domain.setChannelId).mockResolvedValueOnce(undefined);
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce(backendConfig as any);

            // Act
            await useAmpStore.getState().setChannelById(1);

            // Assert
            expect(domain.setChannelId).toHaveBeenCalledWith({channelId: 1});
            expect(domain.getAmpConfig).toHaveBeenCalledTimes(1);
            expect(useAmpStore.getState().current_channel).toBe(1);
        });

        it("addChannel calls backend", async () => {
            // Arrange
            vi.mocked(domain.addChannel).mockResolvedValueOnce(undefined);

            // Act
            await useAmpStore.getState().addChannel("Crunch");

            // Assert
            expect(domain.addChannel).toHaveBeenCalledWith({channelName: "Crunch"});
        });

        it("addChannelFromBackend appends new channel", async () => {
            // Arrange
            const dto = {id: 5, name: "Rhythm", gain: 1, tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5}, volume: 1, effect_chain: []};

            // Act
            await useAmpStore.getState().addChannelFromBackend(dto as any);

            // Assert
            expect(useAmpStore.getState().channels.some((c) => c.id === 5)).toBe(true);
            expect(useAmpStore.getState().current_channel).toBe(5);
        });

        it("removeChannel refreshes config after backend call", async () => {
            // Arrange
            vi.mocked(domain.removeChannel).mockResolvedValueOnce(undefined);
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                master_volume: 1,
                is_active: false,
                current_channel: 0,
                channels: [useAmpStore.getState().channels[0]],
            } as any);

            // Act
            await useAmpStore.getState().removeChannel(1);

            // Assert
            expect(domain.removeChannel).toHaveBeenCalledWith({channelId: 1});
            expect(useAmpStore.getState().channels).toHaveLength(1);
        });

        it("setMasterVolume updates local state and calls backend", () => {
            // Arrange
            const setMasterVolume = useAmpStore.getState().setMasterVolume;

            // Act
            setMasterVolume(0.65);

            // Assert
            expect(useAmpStore.getState().master_volume).toBe(0.65);
            expect(domain.setMasterVolume).toHaveBeenCalledWith({masterVolume: 0.65});
        });

        it("setIsActive updates local state and calls backend toggle", () => {
            // Arrange
            const setIsActive = useAmpStore.getState().setIsActive;

            // Act
            setIsActive(true);

            // Assert
            expect(useAmpStore.getState().is_active).toBe(true);
            expect(domain.toggleOnOff).toHaveBeenCalledWith({isOn: true});
        });

        it("setGain/setVolume/setBass/setMiddle/setTreble update only current channel and call backend", () => {
            // Arrange
            const beforeOther = useAmpStore.getState().channels[1];

            // Act
            useAmpStore.getState().setGain(1.4);
            useAmpStore.getState().setVolume(0.77);
            useAmpStore.getState().setBass(0.2);
            useAmpStore.getState().setMiddle(0.3);
            useAmpStore.getState().setTreble(0.4);

            // Assert
            const current = useAmpStore.getState().channels.find((c) => c.id === 0)!;
            const other = useAmpStore.getState().channels.find((c) => c.id === 1)!;
            expect(current.gain).toBe(1.4);
            expect(current.volume).toBe(0.77);
            expect(current.tone_stack).toEqual({bass: 0.2, middle: 0.3, treble: 0.4});
            expect(other).toEqual(beforeOther);
            expect(domain.setGain).toHaveBeenCalledWith({gain: 1.4});
            expect(domain.setVolume).toHaveBeenCalledWith({volume: 0.77});
            expect(domain.setBass).toHaveBeenCalledWith({bass: 0.2});
            expect(domain.setMiddle).toHaveBeenCalledWith({middle: 0.3});
            expect(domain.setTreble).toHaveBeenCalledWith({treble: 0.4});
        });

        it("updateEffectActiveState updates matching effect only", () => {
            // Arrange
            const effectId = 10;

            // Act
            useAmpStore.getState().updateEffectActiveState(effectId, false);

            // Assert
            const effects = useAmpStore.getState().channels[0].effect_chain;
            const target = effects.find((e: any) => e.data.id === effectId);
            expect(target?.data.is_active).toBe(false);
        });

        it("updateHcDistortionParams and updateDelayParams patch matching effects", () => {
            // Arrange
            const store = useAmpStore.getState();

            // Act
            store.updateHcDistortionParams(10, {threshold: 0.8, level: 0.9});
            store.updateDelayParams(11, {delay_time: 300, level: 0.2});

            // Assert
            const effects = useAmpStore.getState().channels[0].effect_chain as any[];
            expect(effects.find((e) => e.data.id === 10)?.data).toMatchObject({threshold: 0.8, level: 0.9});
            expect(effects.find((e) => e.data.id === 11)?.data).toMatchObject({delay_time: 300, level: 0.2});
        });

        it("removeEffect refreshes config after backend call", async () => {
            // Arrange
            vi.mocked(domain.removeEffect).mockResolvedValueOnce(undefined);
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                ...useAmpStore.getState(),
                channels: [{...useAmpStore.getState().channels[0], effect_chain: [delayEffect]}],
            } as any);

            // Act
            await useAmpStore.getState().removeEffect(10);

            // Assert
            expect(domain.removeEffect).toHaveBeenCalledWith({effectId: 10});
            expect(useAmpStore.getState().channels[0].effect_chain).toHaveLength(1);
        });

        it("addEffect calls backend and refreshes config", async () => {
            // Arrange
            const newEffect = {
                kind: "Delay",
                data: {id: 99, name: "Echo", is_active: true, delay_time: 180, level: 0.5},
            } as any;
            vi.mocked(domain.addEffect).mockResolvedValueOnce(undefined);
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                ...useAmpStore.getState(),
                channels: [{...useAmpStore.getState().channels[0], effect_chain: [...useAmpStore.getState().channels[0].effect_chain, newEffect]}],
            } as any);

            // Act
            await useAmpStore.getState().addEffect(newEffect);

            // Assert
            expect(domain.addEffect).toHaveBeenCalledWith({effectDto: newEffect});
            expect(useAmpStore.getState().channels[0].effect_chain).toHaveLength(3);
        });

        it("moveEffect reorders effects in current channel", async () => {
            // Arrange
            const before = useAmpStore.getState().channels[0].effect_chain as any[];
            expect(before[0].data.id).toBe(10);

            // Act
            await useAmpStore.getState().moveEffect(0, 1);

            // Assert
            const after = useAmpStore.getState().channels[0].effect_chain as any[];
            expect(after[0].data.id).toBe(11);
            expect(after[1].data.id).toBe(10);
        });

        it("startEditingChainOrder and cancelEditingChainOrder restore snapshot", () => {
            // Arrange
            useAmpStore.getState().startEditingChainOrder();
            useAmpStore.getState().moveEffect(0, 1);

            // Act
            useAmpStore.getState().cancelEditingChainOrder();

            // Assert
            const restored = useAmpStore.getState().channels[0].effect_chain as any[];
            expect(restored[0].data.id).toBe(10);
            expect(useAmpStore.getState().chain_snapshot).toBeNull();
        });

        it("applyChangesToChainOrder calls backend and clears snapshot", async () => {
            // Arrange
            useAmpStore.setState({chain_snapshot: [...useAmpStore.getState().channels[0].effect_chain] as any});
            vi.mocked(domain.applyEffectOrderChange).mockResolvedValueOnce(undefined);

            // Act
            await useAmpStore.getState().applyChangesToChainOrder();

            // Assert
            expect(domain.applyEffectOrderChange).toHaveBeenCalledTimes(1);
            expect(useAmpStore.getState().chain_snapshot).toBeNull();
        });
    });

    describe("failure_path", () => {
        it("init handles backend failure without mutating state", async () => {
            // Arrange
            const baseline = useAmpStore.getState().is_active;
            const expectedError = new Error("Backend unavailable");
            vi.mocked(domain.getAmpConfig).mockRejectedValueOnce(expectedError);
            const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            await useAmpStore.getState().init();

            // Assert
            expect(consoleErrorSpy).toHaveBeenCalledWith("Failed to fetch init state from Rust:", expectedError);
            expect(useAmpStore.getState().is_active).toBe(baseline);
            consoleErrorSpy.mockRestore();
        });

        it("setChannelById logs error when backend setChannelId fails", async () => {
            // Arrange
            const expectedError = new Error("set channel failed");
            vi.mocked(domain.setChannelId).mockRejectedValueOnce(expectedError);
            const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            await useAmpStore.getState().setChannelById(1);

            // Assert
            expect(consoleErrorSpy).toHaveBeenCalledWith("Failed to set channel index:", expectedError);
            consoleErrorSpy.mockRestore();
        });

        it("addChannel/removeChannel/removeEffect/addEffect handle backend failures", async () => {
            // Arrange
            const err = new Error("request failed");
            vi.mocked(domain.addChannel).mockRejectedValueOnce(err);
            vi.mocked(domain.removeChannel).mockRejectedValueOnce(err);
            vi.mocked(domain.removeEffect).mockRejectedValueOnce(err);
            vi.mocked(domain.addEffect).mockRejectedValueOnce(err);
            const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            await useAmpStore.getState().addChannel("Bad");
            await useAmpStore.getState().removeChannel(9);
            await useAmpStore.getState().removeEffect(10);
            await useAmpStore.getState().addEffect(delayEffect as any);

            // Assert
            expect(consoleErrorSpy).toHaveBeenCalled();
            consoleErrorSpy.mockRestore();
        });

        it("addChannelFromBackend replaces existing channel instead of duplicating", async () => {
            // Arrange
            const existing = useAmpStore.getState().channels[0];
            const updated = {...existing, name: "Renamed"};

            // Act
            await useAmpStore.getState().addChannelFromBackend(updated as any);

            // Assert
            const channels = useAmpStore.getState().channels;
            expect(channels.filter((c) => c.id === existing.id)).toHaveLength(1);
            expect(channels.find((c) => c.id === existing.id)?.name).toBe("Renamed");
        });

        it("updateEffectActiveState and param updates ignore unknown ids", () => {
            // Arrange
            const before = structuredClone(useAmpStore.getState().channels[0].effect_chain as any[]);

            // Act
            useAmpStore.getState().updateEffectActiveState(999, false);
            useAmpStore.getState().updateHcDistortionParams(999, {threshold: 0.1});
            useAmpStore.getState().updateDelayParams(999, {delay_time: 999});

            // Assert
            expect(useAmpStore.getState().channels[0].effect_chain).toEqual(before);
        });

        it("moveEffect does nothing for out-of-range indices", async () => {
            // Arrange
            const before = [...(useAmpStore.getState().channels[0].effect_chain as any[])];

            // Act
            await useAmpStore.getState().moveEffect(-1, 0);
            await useAmpStore.getState().moveEffect(0, 99);

            // Assert
            expect(useAmpStore.getState().channels[0].effect_chain).toEqual(before);
        });

        it("startEditingChainOrder logs warning if current channel is missing", () => {
            // Arrange
            useAmpStore.setState({current_channel: 999});
            const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => undefined);

            // Act
            useAmpStore.getState().startEditingChainOrder();

            // Assert
            expect(warnSpy).toHaveBeenCalledWith("Could not find current channel to snapshot.");
            warnSpy.mockRestore();
        });

        it("applyChangesToChainOrder returns early when current channel is missing", async () => {
            // Arrange
            useAmpStore.setState({current_channel: 999});
            const errorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            await useAmpStore.getState().applyChangesToChainOrder();

            // Assert
            expect(domain.applyEffectOrderChange).not.toHaveBeenCalled();
            expect(errorSpy).toHaveBeenCalledWith("No active channel found to apply order changes.");
            errorSpy.mockRestore();
        });

        it("applyChangesToChainOrder handles backend failure", async () => {
            // Arrange
            const err = new Error("persist order failed");
            vi.mocked(domain.applyEffectOrderChange).mockRejectedValueOnce(err);
            const errorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            await useAmpStore.getState().applyChangesToChainOrder();

            // Assert
            expect(errorSpy).toHaveBeenCalledWith("Failed to change Effect order:", err);
            errorSpy.mockRestore();
        });
    });
});
