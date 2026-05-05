import {Button, Dialog, DialogActions, DialogContent, DialogTitle, Stack, TextField} from "@mui/material";
import {DropdownSelector} from "../selection/DropdownSelector.tsx";
import {type EffectDto} from "../../domain";
import {useState} from "react";
import {EFFECT_FACTORIES, EFFECT_METADATA} from "../../config/effects";

interface AddEffectDialogProps {
    open: boolean;
    onClose: () => void;
    onCreate: (effect: EffectDto) => void;
}

export type EffectKind = EffectDto["kind"];

export const EFFECT_OPTIONS = Object.entries(EFFECT_METADATA).map(([kind, meta]) => ({
    label: meta.label,
    value: kind as EffectKind,
}));

export function AddEffectDialog({open, onClose, onCreate}: AddEffectDialogProps) {
    const [selectedEffect, setSelectedEffect] = useState<EffectKind | "">("");
    const [name, setName] = useState("");
    const [color, setColor] = useState("#ff4400");

    const handleSelection = (value: string | number) => {
        setSelectedEffect(value as EffectKind);
        console.log("Selected kind:", value);
    };

    const handleCreate = () => {
        if (selectedEffect && name) {
            const defaultData = EFFECT_FACTORIES[selectedEffect]({
                name: name,
                color: color,
            });

            const fullDto: EffectDto = {
                kind: selectedEffect,
                data: defaultData
            } as EffectDto;

            onCreate(fullDto);
            onClose();
        }
    };

    return (
        <Dialog
            open={open}
            onClose={onClose}
            fullWidth
            maxWidth="sm"
        >
            <DialogTitle>New Effect</DialogTitle>

            <DialogContent>
                <Stack direction="column" spacing={2} sx={{paddingY: 2}}>
                    <DropdownSelector label={"Effect Type"} options={EFFECT_OPTIONS} selectedValue={selectedEffect}
                                      onSelectionChange={handleSelection}/>
                    <Stack direction="row" spacing={2}>
                        <TextField
                            label="Name"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            sx={{ width: 450 }}
                            slotProps={{ htmlInput: { maxLength: 15 } }}
                        />
                        <TextField
                            type="color"
                            label="Color"
                            value={color}
                            onChange={(e) => setColor(e.target.value)}
                            sx={{ width: 100 }}
                        />
                    </Stack>
                </Stack>
            </DialogContent>

            <DialogActions>
                <Button onClick={onClose}>Cancel</Button>
                <Button
                    variant="contained"
                    disabled={!selectedEffect || !name}
                    onClick={handleCreate}
                >
                    Create
                </Button>
            </DialogActions>
        </Dialog>
    );
}