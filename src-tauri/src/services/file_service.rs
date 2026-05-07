use crate::infrastructure::file_loader::FileLoaderTrait;
use std::path::PathBuf;
use tracing::{info, warn};

const DEFAULT_IR_DIRECTORY_NAME: &str = "default_ir";
const RESOURCES_DIRECTORY_NAME: &str = "resources";

/// Service for IR profile file discovery.
pub struct FileService {
    file_loader: Box<dyn FileLoaderTrait>,
    resource_root: PathBuf,
}

impl FileService {
    pub fn new(file_loader: Box<dyn FileLoaderTrait>, resource_root: PathBuf) -> Self {
        Self {
            file_loader,
            resource_root,
        }
    }

    pub fn get_all_ir_profiles(&self) -> Result<Vec<String>, String> {
        let ir_directory = self.resolve_ir_directory()?;
        let file_names = self.file_loader.list_ir_profile_file_names(&ir_directory)?;

        if file_names.is_empty() {
            return Err(format!(
                "No IR profiles found in '{}'",
                ir_directory.display()
            ));
        }

        Ok(file_names
            .into_iter()
            .collect())
    }

    fn resolve_ir_directory(&self) -> Result<PathBuf, String> {
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

