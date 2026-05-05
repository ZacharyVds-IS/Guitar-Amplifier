use crate::domain::dto::amp_config_dto::AmpConfigDto;
use crate::domain::dto::channel_dto::ChannelDto;
use crate::infrastructure::persistence::amp_config_persistence_trait::AmpConfigPersistence;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// File-based amp-config repository backed by a single JSON document.
///
/// The repository stores one full amplifier snapshot at `config_path`. It is
/// intentionally simple: every save overwrites the entire file and every load
/// reads the whole document into memory.
///
/// This implementation is useful while the configuration remains relatively
/// small and the project does not yet need querying, concurrency control, or
/// schema migrations provided by a database.
pub struct JsonFileAmpConfigRepository {
    config_path: PathBuf,
}

/// Persistence-only representation of the amplifier configuration.
///
/// This struct deliberately differs from [`AmpConfigDto`]: it excludes
/// `is_active`, because loopback state is considered runtime-only and the app
/// should always restart in an "off" state.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedAmpConfig {
    master_volume: f32,
    channels: Vec<ChannelDto>,
    current_channel: u32,
}

impl From<&AmpConfigDto> for PersistedAmpConfig {
    fn from(config: &AmpConfigDto) -> Self {
        Self {
            master_volume: config.master_volume,
            channels: config.channels.clone(),
            current_channel: config.current_channel,
        }
    }
}

impl From<PersistedAmpConfig> for AmpConfigDto {
    fn from(config: PersistedAmpConfig) -> Self {
        Self {
            master_volume: config.master_volume,
            is_active: false,
            channels: config.channels,
            current_channel: config.current_channel,
        }
    }
}

impl JsonFileAmpConfigRepository {
    /// Creates a JSON repository that reads from and writes to `config_path`.
    ///
    /// The path is not validated eagerly. Missing parent directories are
    /// created on the first successful `save` call.
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }
}

impl AmpConfigPersistence for JsonFileAmpConfigRepository {
    /// Loads and deserializes the persisted JSON file.
    ///
    /// Behavior summary:
    /// - missing file -> `Ok(None)`
    /// - unreadable file -> `Err(String)`
    /// - invalid JSON -> `Err(String)`
    /// - valid JSON -> `Ok(Some(AmpConfigDto))`
    fn load(&self) -> Result<Option<AmpConfigDto>, String> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let payload = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read amp config '{}': {e}", self.config_path.display()))?;

        let persisted = serde_json::from_str::<PersistedAmpConfig>(&payload)
            .map_err(|e| format!("Failed to parse amp config JSON '{}': {e}", self.config_path.display()))?;

        Ok(Some(AmpConfigDto::from(persisted)))
    }

    /// Serializes the supplied config snapshot and writes it to disk.
    ///
    /// Parent directories are created automatically when necessary. The JSON is
    /// formatted with `to_string_pretty` so it remains reasonably human-readable
    /// during development and debugging.
    fn save(&self, config: &AmpConfigDto) -> Result<(), String> {
        if let Some(parent) = self.config_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "Failed to create config directory '{}': {e}",
                        parent.display()
                    )
                })?;
            }
        }

        let persisted = PersistedAmpConfig::from(config);
        let json = serde_json::to_string_pretty(&persisted)
            .map_err(|e| format!("Failed to serialize amp config: {e}"))?;

        fs::write(&self.config_path, json).map_err(|e| {
            format!(
                "Failed to write amp config '{}': {e}",
                self.config_path.display()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("rustriff-amp-config-{nanos}.json"))
    }

    #[test]
    fn save_then_load_roundtrip_succeeds() {
        let path = unique_test_path();
        let repo = JsonFileAmpConfigRepository::new(path.clone());

        let config = AmpConfigDto {
            master_volume: 0.8,
            is_active: true,
            channels: Vec::new(),
            current_channel: 0,
        };

        repo.save(&config).expect("save should succeed");
        let loaded = repo.load().expect("load should succeed").expect("config should exist");
        let raw_json = fs::read_to_string(path.clone()).expect("saved file should be readable");

        assert!((loaded.master_volume - config.master_volume).abs() < 1e-6);
        assert_eq!(loaded.current_channel, config.current_channel);
        assert!(!loaded.is_active);
        assert!(!raw_json.contains("is_active"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_returns_none_when_file_missing() {
        let path = unique_test_path();
        let repo = JsonFileAmpConfigRepository::new(path);

        let loaded = repo.load().expect("load should succeed");
        assert!(loaded.is_none());
    }
}


