use serde::Serialize;

#[derive(Serialize,Clone)]
pub struct AudioDeviceDto{
    pub name: String,
    pub id: String
}