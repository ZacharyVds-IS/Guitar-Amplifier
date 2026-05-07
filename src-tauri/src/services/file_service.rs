use crate::domain::dto::effect::ir_profile_dto::IrProfileDto;
use crate::infrastructure::file_loader::FileLoaderTrait;
use std::path::PathBuf;
use tracing::{info, warn};

const DEFAULT_IR_DIRECTORY_NAME: &str = "default_ir";
const RESOURCES_DIRECTORY_NAME: &str = "resources";
const IMPULSE_THRESHOLD: f32 = 1e-6;

/// Service for IR profile file discovery.
pub struct FileService {
    file_loader: Box<dyn FileLoaderTrait>,
    resource_root: PathBuf,
    custom_ir_directory: PathBuf,
}

impl FileService {
    pub fn new(
        file_loader: Box<dyn FileLoaderTrait>,
        resource_root: PathBuf,
        custom_ir_directory: PathBuf,
    ) -> Self {
        Self {
            file_loader,
            resource_root,
            custom_ir_directory,
        }
    }

    pub fn get_all_ir_profiles(&self) -> Result<Vec<IrProfileDto>, String> {
        let default_directory = self.resolve_default_ir_directory()?;
        self.file_loader.ensure_directory(&self.custom_ir_directory)?;

        let mut profiles = self
            .file_loader
            .list_ir_profile_file_names(&default_directory)?
            .into_iter()
            .map(|file_name| IrProfileDto {
                label: to_readable_label(&file_name),
                file_name,
                is_custom: false,
                is_in_use: false,
            })
            .collect::<Vec<_>>();

        let custom_profiles = self
            .file_loader
            .list_ir_profile_file_names(&self.custom_ir_directory)?
            .into_iter()
            .map(|file_name| IrProfileDto {
                label: to_readable_label(&file_name),
                file_name,
                is_custom: true,
                is_in_use: false,
            });

        profiles.extend(custom_profiles);
        profiles.sort_by(|a, b| a.label.cmp(&b.label));
        Ok(profiles)
    }

    pub fn save_custom_ir_profile(&self, file_name: &str, file_bytes: &[u8]) -> Result<String, String> {
        let sanitized_file_name = sanitize_wav_file_name(file_name)?;

        self.file_loader.validate_ir_wav_bytes(
            &sanitized_file_name,
            file_bytes,
            IMPULSE_THRESHOLD,
        )?;

        let default_directory = self.resolve_default_ir_directory()?;
        let default_path = default_directory.join(&sanitized_file_name);
        if default_path.exists() {
            return Err(format!(
                "An IR named '{}' already exists in defaults",
                sanitized_file_name
            ));
        }

        self.file_loader.ensure_directory(&self.custom_ir_directory)?;
        let custom_path = self.custom_ir_directory.join(&sanitized_file_name);
        self.file_loader.write_file_bytes(&custom_path, file_bytes)?;

        Ok(sanitized_file_name)
    }

    pub fn remove_custom_ir_profile(&self, file_name: &str) -> Result<(), String> {
        let sanitized_file_name = sanitize_wav_file_name(file_name)?;
        let custom_path = self.custom_ir_directory.join(&sanitized_file_name);

        if !custom_path.exists() {
            return Err(format!(
                "Custom IR '{}' does not exist",
                sanitized_file_name
            ));
        }

        self.file_loader.remove_file(&custom_path)
    }

    pub fn default_ir_directory(&self) -> Result<PathBuf, String> {
        self.resolve_default_ir_directory()
    }

    pub fn custom_ir_directory(&self) -> PathBuf {
        self.custom_ir_directory.clone()
    }

    fn resolve_default_ir_directory(&self) -> Result<PathBuf, String> {
        let mut candidates = if cfg!(debug_assertions) {
            vec![
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join(RESOURCES_DIRECTORY_NAME)
                    .join(DEFAULT_IR_DIRECTORY_NAME),
            ]
        } else {
            vec![
                self.resource_root.join(DEFAULT_IR_DIRECTORY_NAME),
                self.resource_root
                    .join(RESOURCES_DIRECTORY_NAME)
                    .join(DEFAULT_IR_DIRECTORY_NAME),
            ]
        };

        candidates.dedup();

        for candidate in &candidates {
            if candidate.is_dir() {
                info!("Using default IR directory: {}", candidate.display());
                return Ok(candidate.clone());
            }
            warn!("Skipping missing IR directory candidate: {}", candidate.display());
        }

        let searched = candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        Err(format!("Could not locate default IR directory. Searched: {searched}"))
    }
}

fn sanitize_wav_file_name(file_name: &str) -> Result<String, String> {
    let trimmed = file_name.trim();

    if trimmed.is_empty() {
        return Err("IR file name cannot be empty".to_string());
    }

    if trimmed.contains('\\') || trimmed.contains('/') || trimmed.contains("..") {
        return Err("Invalid IR file name".to_string());
    }

    if !trimmed.to_ascii_lowercase().ends_with(".wav") {
        return Err("Only .wav IR files are supported".to_string());
    }

    Ok(trimmed.to_string())
}

fn to_readable_label(file_name: &str) -> String {
    file_name
        .trim_end_matches(".wav")
        .trim_end_matches(".WAV")
        .replace(['-', '_'], " ")
}

