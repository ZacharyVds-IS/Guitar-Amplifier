import {FormControl, InputLabel, MenuItem, Select, SelectChangeEvent, Typography} from "@mui/material";

interface DropdownProps {
    title: string;
    label: string;
    options: { label: string; value: string | number }[];
    selectedValue: string | number;
    onSelectionChange: (value: string) => void;
}

export function DropdownSelector({title, label, options, selectedValue, onSelectionChange}: DropdownProps) {
    const selectId = `${title}-${label}`.toLowerCase().replace(/\s+/g, "-");
    const labelId = `${selectId}-label`;

    const handleChange = (event: SelectChangeEvent<string | number>) => {
        onSelectionChange(event.target.value as string);
    };
    return (
        <>
            <Typography variant="h6" gutterBottom>
                {title}
            </Typography>

            <FormControl fullWidth>
                <InputLabel id={labelId}>{label}</InputLabel>
                <Select
                    labelId={labelId}
                    id={selectId}
                    value={selectedValue}
                    label={label}
                    onChange={handleChange}
                >
                    {options.map((option) => (
                        <MenuItem key={option.value} value={option.value}>
                            {option.label}
                        </MenuItem>
                    ))}
                </Select>
            </FormControl>
        </>
    )
        ;
}