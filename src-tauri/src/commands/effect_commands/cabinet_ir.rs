use crate::services::file_service::FileService;

#[tauri::command]
pub fn get_all_ir_profiles(file_service: tauri::State<FileService>) -> Result<Vec<String>, String> {
	file_service.get_all_ir_profiles()
}

