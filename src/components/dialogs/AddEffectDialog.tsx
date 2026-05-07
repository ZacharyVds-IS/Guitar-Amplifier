import {Button, Dialog, DialogActions, DialogContent, DialogTitle} from "@mui/material";
import {type EffectDto, getAllIrProfiles} from "../../domain";
import {useEffect, useState} from "react";
import {
    CABINET_CUSTOM_IR_VALUE,
    DEFAULT_CABINET_IR_FILE,
    EFFECT_FACTORIES,
    type EffectKind,
} from "../../config/effects";
import {useForm} from "react-hook-form";
import {zodResolver} from "@hookform/resolvers/zod";
import {
    type AddEffectFormValues,
    addEffectSchema,
    DEFAULT_ADD_EFFECT_FORM_VALUES,
    isEffectKind,
} from "../../config/schemas/addEffectSchema";
import {AddEffectFormFields} from "../forms/AddEffectFormFields.tsx";

interface AddEffectDialogProps {
    open: boolean;
    onClose: () => void;
    onCreate: (effect: EffectDto) => void;
}

export function AddEffectDialog({open, onClose, onCreate}: AddEffectDialogProps) {
    const [cabinetIrOptions, setCabinetIrOptions] = useState<{ label: string; value: string }[]>([
        {label: "Custom IR file", value: CABINET_CUSTOM_IR_VALUE},
    ]);

    const {control, register, handleSubmit, reset, formState: {errors, isValid},} = useForm<AddEffectFormValues>({
        resolver: zodResolver(addEffectSchema),
        mode: "onChange",
        defaultValues: DEFAULT_ADD_EFFECT_FORM_VALUES,
    });

    useEffect(() => {
        if (!open) {
            reset(DEFAULT_ADD_EFFECT_FORM_VALUES);
        }
    }, [open, reset]);

    useEffect(() => {
        if (!open) {
            return;
        }

        let active = true;

        const loadIrProfiles = async () => {
            try {
                const profiles = await getAllIrProfiles();
                if (!active) {
                    return;
                }

                const dynamicOptions = profiles.map((profile) => ({
                    label: toReadableIrLabel(profile),
                    value: profile,
                }));

                setCabinetIrOptions([
                    ...dynamicOptions,
                    {label: "Custom IR file", value: CABINET_CUSTOM_IR_VALUE},
                ]);
            } catch (error) {
                console.error("Failed to load cabinet IR profiles:", error);
                if (active) {
                    setCabinetIrOptions([{label: "Custom IR file", value: CABINET_CUSTOM_IR_VALUE}]);
                }
            }
        };

        loadIrProfiles();

        return () => {
            active = false;
        };
    }, [open]);

    const handleDialogClose = () => {
        reset(DEFAULT_ADD_EFFECT_FORM_VALUES);
        onClose();
    };

    const handleCreate = (values: AddEffectFormValues) => {
        if (!isEffectKind(values.selectedEffect)) {
            return;
        }

        const effectKind: EffectKind = values.selectedEffect;

        const selectedCabinetIrFile =
            effectKind === "Cabinet" && values.cabinetIrChoice !== CABINET_CUSTOM_IR_VALUE
                ? values.cabinetIrChoice
                : DEFAULT_CABINET_IR_FILE;

        const defaultData = EFFECT_FACTORIES[effectKind]({
            name: values.name.trim(),
            color: values.color,
            cabinetIrFilePath: selectedCabinetIrFile,
        });

        const fullDto: EffectDto = {
            kind: effectKind,
            data: defaultData
        } as EffectDto;

        onCreate(fullDto);
        handleDialogClose();
    };

    return (
        <Dialog
            open={open}
            onClose={handleDialogClose}
            fullWidth
            maxWidth="sm"
        >
            <DialogTitle>New Effect</DialogTitle>

            <DialogContent>
                <AddEffectFormFields
                    control={control}
                    register={register}
                    errors={errors}
                    cabinetIrOptions={cabinetIrOptions}
                />
            </DialogContent>

            <DialogActions>
                <Button onClick={handleDialogClose}>Cancel</Button>
                <Button
                    variant="contained"
                    disabled={!isValid}
                    onClick={handleSubmit(handleCreate)}
                >
                    Create
                </Button>
            </DialogActions>
        </Dialog>
    );
}

function toReadableIrLabel(fileName: string): string {
    return fileName
        .replace(/\.[^/.]+$/, "")
        .replace(/[-_]+/g, " ")
        .trim();
}
