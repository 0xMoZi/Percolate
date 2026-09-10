// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use percolate_core::build_tree_from_leaves;

#[tauri::command]
fn compute_root(leaves_hex: Vec<String>) -> Result<String, String> {
    let leaves: Vec<[u8; 32]> = leaves_hex
        .iter()
        .map(|s| {
            let bytes = hex::decode(s.trim_start_matches("0x")).map_err(|e| e.to_string())?;
            bytes
                .try_into()
                .map_err(|_| "leaf harus 32 byte".to_string())
        })
        .collect::<Result<_, String>>()?;

    let (_, root) = build_tree_from_leaves(&leaves);
    Ok(format!("0x{}", hex::encode(root)))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, compute_root])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
