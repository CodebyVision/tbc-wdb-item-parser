// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub mod itemcache_export;

#[tauri::command]
fn export_itemcache_to_item_template_sql(itemcache_path: String, output_sql_path: String) -> Result<usize, String> {
    itemcache_export::export_itemcache_to_cmangos_item_template_sql(
        std::path::Path::new(&itemcache_path),
        std::path::Path::new(&output_sql_path),
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, export_itemcache_to_item_template_sql])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
