import {beforeEach, describe, expect, it} from "vitest";
import {useUIStore} from "../../state/UIStore";

function resetUIStore() {
    useUIStore.setState({
        developerMode: false,
        selectedInputId: "",
        selectedOutputId: "",
    });
}

describe("UIStore", () => {
    beforeEach(() => {
        resetUIStore();
    });

    describe("success_path", () => {
        it("setDeveloperMode toggles developer mode", () => {
            // Arrange
            const setDeveloperMode = useUIStore.getState().setDeveloperMode;

            // Act
            setDeveloperMode(true);

            // Assert
            expect(useUIStore.getState().developerMode).toBe(true);
        });

        it("setSelectedInputId updates selected input id", () => {
            // Arrange
            const setSelectedInputId = useUIStore.getState().setSelectedInputId;

            // Act
            setSelectedInputId("input-123");

            // Assert
            expect(useUIStore.getState().selectedInputId).toBe("input-123");
        });

        it("setSelectedOutputId updates selected output id", () => {
            // Arrange
            const setSelectedOutputId = useUIStore.getState().setSelectedOutputId;

            // Act
            setSelectedOutputId("output-456");

            // Assert
            expect(useUIStore.getState().selectedOutputId).toBe("output-456");
        });
    });

    describe("failure_path", () => {
        it("accepts empty ids without throwing (clear selection path)", () => {
            // Arrange
            const setSelectedInputId = useUIStore.getState().setSelectedInputId;
            const setSelectedOutputId = useUIStore.getState().setSelectedOutputId;

            // Act
            setSelectedInputId("");
            setSelectedOutputId("");

            // Assert
            expect(useUIStore.getState().selectedInputId).toBe("");
            expect(useUIStore.getState().selectedOutputId).toBe("");
        });
    });
});

