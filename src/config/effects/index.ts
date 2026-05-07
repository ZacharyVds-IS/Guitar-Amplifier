import {CabinetDto, type EffectDto, HcDistortionDto} from "../../domain";

export type EffectKind = EffectDto["kind"];

type EffectFactoryMap = {
    [K in EffectKind]: (params: { name: string; color: string; cabinetIrFilePath?: string }) => Extract<EffectDto, { kind: K }>['data'];
};

export const DEFAULT_CABINET_IR_FILE = "info-support-halway.wav";

export const EFFECT_METADATA: Record<EffectKind, { label: string }> = {
    HCDistortion: {label: "Hard-Clipping Distortion"},
    Cabinet: {label:"Cabinet Simulation"}
};

export const CABINET_CUSTOM_IR_VALUE = "__CUSTOM_FILE__";


export const EFFECT_FACTORIES: EffectFactoryMap = {
    HCDistortion: ({ name, color }): HcDistortionDto => ({
        id: 0, // Is set to the correct value in the backend
        name,
        color,
        is_active: false,
        threshold: 1,
        level: 0,
    }),
    Cabinet: ({name,color,cabinetIrFilePath}): CabinetDto => ({
        id:0,
        name,
        color,
        is_active:false,
        ir_file_path: cabinetIrFilePath ?? DEFAULT_CABINET_IR_FILE,
    })
};