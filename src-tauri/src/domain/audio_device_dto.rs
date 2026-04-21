use serde::Serialize;

#[derive(Serialize,Clone)]
pub struct AudioDeviceDto{
    pub id: String,
    pub name: String
}