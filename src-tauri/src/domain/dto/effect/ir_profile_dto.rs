use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IrProfileDto {
    pub file_name: String,
    pub label: String,
    pub is_custom: bool,
    pub is_in_use: bool,
}

