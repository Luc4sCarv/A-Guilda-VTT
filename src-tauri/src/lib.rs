mod mesa;

#[tauri::command]
fn core_status() -> String {
    format!("{} classes carregadas", tormenta20_core::base_ruleset().classes.len())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(mesa::MesaState::default())
        .invoke_handler(tauri::generate_handler![
            core_status,
            mesa::migrate_legacy,
            mesa::list_characters,
            mesa::save_character,
            mesa::delete_character,
            mesa::list_rooms,
            mesa::room_cover,
            mesa::import_room_cover,
            mesa::delete_room,
            mesa::list_master_assets,
            mesa::create_scene,
            mesa::save_threat,
            mesa::save_document,
            mesa::delete_document,
            mesa::import_asset,
            mesa::update_map,
            mesa::activate_map,
            mesa::start_room,
            mesa::stop_room,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar A Guilda VTT");
}
