use percolate_core::{build_tree_from_leaves, leaf_from_secret};
use rand::Rng;
use serde::Serialize;

#[derive(Serialize)]
struct GeneratedTaker {
    index: usize,
    secret: String,
    leaf: String,
}

#[derive(Serialize)]
struct AllowlistResult {
    root: String,
    takers: Vec<GeneratedTaker>,
}

#[tauri::command]
fn generate_allowlist(n: usize) -> AllowlistResult {
    let mut rng = rand::rng();
    let mut secrets: Vec<[u8; 32]> = Vec::with_capacity(n);
    for _ in 0..n {
        let mut secret = [0u8; 32];
        rng.fill_bytes(&mut secret);
        secrets.push(secret);
    }

    let leaves: Vec<[u8; 32]> = secrets.iter().map(leaf_from_secret).collect();
    let (_, root) = build_tree_from_leaves(&leaves);

    let takers = secrets
        .iter()
        .zip(leaves.iter())
        .enumerate()
        .map(|(i, (s, l))| GeneratedTaker {
            index: i,
            secret: format!("0x{}", hex::encode(s)),
            leaf: format!("0x{}", hex::encode(l)),
        })
        .collect();

    AllowlistResult {
        root: format!("0x{}", hex::encode(root)),
        takers,
    }
}

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
        .invoke_handler(tauri::generate_handler![
            greet,
            compute_root,
            generate_allowlist
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
