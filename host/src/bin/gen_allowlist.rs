use percolate_core::leaf_from_secret;
use rand::Rng;
use std::fs;

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);

    let mut rng = rand::rng();

    let mut secrets: Vec<[u8; 32]> = Vec::with_capacity(n);
    for _ in 0..n {
        let mut secret = [0u8; 32];
        rng.fill_bytes(&mut secret);
        secrets.push(secret);
    }

    let leaves: Vec<[u8; 32]> = secrets.iter().map(leaf_from_secret).collect();

    let leaves_hex: Vec<String> = leaves
        .iter()
        .map(|l| format!("0x{}", hex::encode(l)))
        .collect();
    let allowlist_json = serde_json::json!({"leaves": leaves_hex});
    fs::write(
        "allowlist.json",
        serde_json::to_string_pretty(&allowlist_json).unwrap(),
    )
    .expect("failed write allowlist.json");

    for (i, secret) in secrets.iter().enumerate() {
        let secret_json = serde_json::json!({
            "index": i,
            "secret": format!("0x{}", hex::encode(secret)),
        });
        let filename = format!("taker_secret_{i}.json");
        fs::write(
            &filename,
            serde_json::to_string_pretty(&secret_json).unwrap(),
        )
        .expect("failed to write taker secret file");
        println!("wrote {filename}");
    }

    println!("wrote allowlist.json ({n} leaves)");
}
