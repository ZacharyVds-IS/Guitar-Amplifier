import {FormControl, InputLabel, MenuItem, Select, SelectChangeEvent, Typography} from "@mui/material";

interface DropdownProps {
    title: string;
    label: string;
    options: { label: string; value: string | number }[];
    selectedValue: string | number;
    onSelectionChange: (value: string) => void;
}

export function DropdownSelector({title, label, options, selectedValue, onSelectionChange}: DropdownProps) {
    const handleChange = (event: SelectChangeEvent<string | number>) => {
        onSelectionChange(event.target.value as string);
    };
    return (
        <>
            <Typography variant="h6" gutterBottom>
                {title}
            </Typography>

            <FormControl fullWidth>
                <InputLabel id="simple-select-label">{label}</InputLabel>
                <Select
                    labelId="simple-select-label"
                    id="simple-select"
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