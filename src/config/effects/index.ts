import {CabinetDto, type EffectDto, HcDistortionDto} from "../../domain";
import {EffectKind} from "../../components/dialogs/AddEffectDialog.tsx";

type EffectFactoryMap = {
    [K in EffectKind]: (params: { name: string; color: string }) => Extract<EffectDto, { kind: K }>["data"];
};

export const EFFECT_METADATA: Record<EffectKind, { label: string }> = {
    HCDistortion: {label: "Hard-Clipping Distortion"},
    Cabinet: {label:"Cabinet Simulation"}
};

export const EFFECT_FACTORIES: EffectFactoryMap = {
    HCDistortion: ({ name, color }): HcDistortionDto => ({
        id: 0, // Is set to the correct value in the backend
        name,
        color,
        is_active: false,
        threshold: 1,
        level: 0,
    }),
    Cabinet: ({name,color}): CabinetDto => ({
        id:0,
        name,
        color,
        is_active:false
    })
};