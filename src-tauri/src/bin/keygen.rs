//! Offline license key generator for SEO Insikt.
//!
//! Run from src-tauri/:
//!
//!   cargo run --bin keygen -- issue --tier premium --customer "order-123" --expires-days 365
//!   cargo run --bin keygen -- generate-keys
//!
//! `generate-keys` writes private_key.bin and public_key.bin to the current directory.
//! Keep private_key.bin secret — never commit it. public_key.bin is embedded in the app.
//!
//! `issue` reads private_key.bin from the current directory and outputs a key string
//! ready to paste into the app.

use base64::Engine;
use ed25519_dalek::Signer;
use std::path::Path;

const KEY_PREFIX: &str = "SEOINSIKT-";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LicenseData {
    key: String,
    machine_id: String,
    tier: String,
    tier_version: TierVersion,
    tier_meta: Option<serde_json::Value>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    issued_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
struct TierVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SignedLicense {
    data: LicenseData,
    signature: String,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  keygen generate-keys");
        eprintln!("  keygen issue --tier <premium|free> --customer <string> [--expires-days <n>]");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "generate-keys" => cmd_generate_keys(),
        "issue" => cmd_issue(&args[2..]),
        other => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}

fn cmd_generate_keys() {
    use rand::rngs::OsRng;
    let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    std::fs::write("private_key.bin", signing_key.to_bytes()).expect("write private_key.bin");
    std::fs::write("public_key.bin", verifying_key.to_bytes()).expect("write public_key.bin");

    println!("Keys written:");
    println!("  private_key.bin  — keep secret, never commit");
    println!("  public_key.bin   — embed in app (src-tauri/public_key.bin)");
}

fn cmd_issue(args: &[String]) {
    let mut tier = String::new();
    let mut customer = String::new();
    let mut expires_days: Option<i64> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--tier" => { i += 1; tier = args[i].clone(); }
            "--customer" => { i += 1; customer = args[i].clone(); }
            "--expires-days" => { i += 1; expires_days = Some(args[i].parse().expect("--expires-days must be a number")); }
            other => { eprintln!("Unknown flag: {other}"); std::process::exit(1); }
        }
        i += 1;
    }

    if tier.is_empty() || customer.is_empty() {
        eprintln!("--tier and --customer are required");
        std::process::exit(1);
    }

    let tier_str = match tier.to_lowercase().as_str() {
        "premium" => "Premium",
        "free" => "Free",
        other => { eprintln!("Unknown tier: {other}. Use premium or free"); std::process::exit(1); }
    };

    let private_key_path = Path::new("private_key.bin");
    if !private_key_path.exists() {
        eprintln!("private_key.bin not found. Run `keygen generate-keys` first.");
        std::process::exit(1);
    }

    let key_bytes: [u8; 32] = std::fs::read(private_key_path)
        .expect("read private_key.bin")
        .try_into()
        .expect("private_key.bin must be exactly 32 bytes");

    let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_bytes);

    let expires_at = expires_days.map(|d| chrono::Utc::now() + chrono::Duration::days(d));

    let data = LicenseData {
        key: customer.clone(),
        machine_id: "*".to_string(),
        tier: tier_str.to_string(),
        tier_version: TierVersion { major: 1, minor: 0, patch: 0 },
        tier_meta: None,
        expires_at,
        issued_at: chrono::Utc::now(),
    };

    let data_json = serde_json::to_string(&data).expect("serialize");
    let signature = signing_key.sign(data_json.as_bytes());
    let signed = SignedLicense { data, signature: hex::encode(signature.to_bytes()) };
    let json = serde_json::to_string(&signed).expect("serialize signed");
    let key = format!("{KEY_PREFIX}{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json));

    println!("\n=== SEO Insikt License Key ===");
    println!("Customer : {customer}");
    println!("Tier     : {tier_str}");
    if let Some(days) = expires_days {
        println!("Expires  : {} days from now", days);
    } else {
        println!("Expires  : never");
    }
    println!("\n{key}\n");
}
