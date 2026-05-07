use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::dto::effect::ir_profile_dto::IrProfileDto;
use crate::services::audio_service::AudioService;
use crate::services::file_service::FileService;
use std::collections::HashSet;
use std::sync::Mutex;
use tracing::{info, warn};

#[tauri::command]
pub fn get_all_ir_profiles(
	file_service: tauri::State<FileService>,
	audio_service: tauri::State<Mutex<AudioService>>,
) -> Result<Vec<IrProfileDto>, String> {
	let used_profiles = used_ir_profiles(&audio_service).map_err(|err| {
		warn!("get_all_ir_profiles failed while reading used profiles: {err}");
		err
	})?;
	let mut profiles = file_service.get_all_ir_profiles().map_err(|err| {
		warn!("get_all_ir_profiles failed while reading profile inventory: {err}");
		err
	})?;

	for profile in &mut profiles {
		profile.is_in_use = used_profiles.contains(&profile.file_name);
	}

	Ok(profiles)
}

#[tauri::command]
pub fn upload_ir_profile(
	file_service: tauri::State<FileService>,
	file_name: String,
	file_bytes: Vec<u8>,
) -> Result<String, String> {
	info!(
		"Uploading custom IR profile '{}' ({} bytes)",
		file_name,
		file_bytes.len()
	);
	file_service.save_custom_ir_profile(&file_name, &file_bytes).map_err(|err| {
		warn!("upload_ir_profile failed for '{}': {err}", file_name);
		err
	})
}

#[tauri::command]
pub fn remove_ir_profile(
	file_service: tauri::State<FileService>,
	audio_service: tauri::State<Mutex<AudioService>>,
	file_name: String,
) -> Result<(), String> {
	let profiles = file_service.get_all_ir_profiles().map_err(|err| {
		warn!("remove_ir_profile failed while reading profile inventory: {err}");
		err
	})?;
	let profile = profiles
		.iter()
		.find(|p| p.file_name == file_name)
		.ok_or_else(|| format!("IR profile '{}' not found", file_name))?;

	if !profile.is_custom {
		return Err("Default IR profiles cannot be removed".to_string());
	}

	let used_profiles = used_ir_profiles(&audio_service).map_err(|err| {
		warn!("remove_ir_profile failed while checking chain usage: {err}");
		err
	})?;
	if used_profiles.contains(&file_name) {
		return Err(format!(
			"IR profile '{}' is currently used by an effect chain",
			file_name
		));
	}

	file_service.remove_custom_ir_profile(&file_name).map_err(|err| {
		warn!("remove_ir_profile failed for '{}': {err}", file_name);
		err
	})
}

fn used_ir_profiles(audio_service: &tauri::State<Mutex<AudioService>>) -> Result<HashSet<String>, String> {
	let service = audio_service
		.lock()
		.map_err(|_| "Failed to lock audio service".to_string())?;

	let mut used = HashSet::new();
	for channel in service.channels().iter() {
		let effect_chain = channel.effect_chain();
		let chain = effect_chain
			.lock()
			.map_err(|_| "Failed to lock effect chain".to_string())?;

		for effect in chain.iter() {
			if let EffectDto::Cabinet(cabinet) = effect.to_dto() {
				used.insert(cabinet.ir_file_path);
			}
		}
	}

	Ok(used)
}

