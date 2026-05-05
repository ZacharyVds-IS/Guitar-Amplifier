use crate::domain::dto::amp_config_dto::AmpConfigDto;

pub trait AmpConfigPersistence: Send + Sync {
    fn load(&self) -> Result<Option<AmpConfigDto>, String>;
    fn save(&self, config: &AmpConfigDto) -> Result<(), String>;
}

