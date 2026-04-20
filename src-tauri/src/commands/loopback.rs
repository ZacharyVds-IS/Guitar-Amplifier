use crate::services::audioservice;

#[tauri::command]
pub(crate) fn start_loopback() {
    audioservice::start_loopback();
}