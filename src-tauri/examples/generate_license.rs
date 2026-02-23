use app::domain::licensing::{LicenseData, SignedLicense};
use app::domain::permissions::LicenseTier;
use app::domain::TierVersion;
use ed25519_dalek::{Signer, SigningKey};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- License Generation Utility ---");
    println!("This tool generates a signed license JSON for the mock licensing service.");
    println!();

    // 1. Get Machine ID

    let current_machine_id = get_machine_id();
    println!("\nMachine ID Selection:");
    println!("1. Use Current Machine ID ({})", current_machine_id);
    println!("2. Enter Custom Machine ID");
    print!("Choice [1]: ");
    io::stdout().flush()?;
    let mut mach_choice = String::new();
    io::stdin().read_line(&mut mach_choice)?;

    let machine_id = match mach_choice.trim() {
        "2" => {
            print!("Enter Custom Machine ID: ");
            io::stdout().flush()?;
            let mut custom_id = String::new();
            io::stdin().read_line(&mut custom_id)?;
            let id = custom_id.trim().to_string();
            if id.is_empty() {
                current_machine_id
            } else {
                id
            }
        }
        _ => current_machine_id,
    };

    // 2. Select Tier
    println!("\nSelect License Tier:");
    println!("1. Premium");
    println!("2. Free");
    print!("Choice [1]: ");
    io::stdout().flush()?;
    let mut tier_choice = String::new();
    io::stdin().read_line(&mut tier_choice)?;
    let tier = match tier_choice.trim() {
        "2" => LicenseTier::Free,
        _ => LicenseTier::Premium,
    };

    // 3. Set Private Key
    println!("\nPrivate Key Selection:");
    println!("1. Use Default Mock Private Key");
    println!("2. Enter Custom Private Key (Hex)");
    print!("Choice [1]: ");
    io::stdout().flush()?;
    let mut key_choice = String::new();
    io::stdin().read_line(&mut key_choice)?;

    let private_key_bytes = match key_choice.trim() {
        "2" => {
            print!("Enter Private Key (Hex): ");
            io::stdout().flush()?;
            let mut hex_key = String::new();
            io::stdin().read_line(&mut hex_key)?;
            let bytes = hex::decode(hex_key.trim())?;
            if bytes.len() != 32 {
                return Err("Private key must be 32 bytes (64 hex characters)".into());
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            arr
        }
        _ => MOCK_PRIVATE_KEY,
    };

    // 4. Generate License Data
    let data = LicenseData {
        key: "MOCK-KEY-".to_string() + &uuid::Uuid::new_v4().to_string().to_uppercase()[..8],
        machine_id: machine_id.clone(),
        tier,
        tier_version: TierVersion::new(1, 0, 0),
        tier_meta: None,
        expires_at: None,
        issued_at: chrono::Utc::now(),
    };

    // 5. Sign Data
    let data_json = serde_json::to_string(&data)?;
    let signing_key = SigningKey::from_bytes(&private_key_bytes);
    let signature = signing_key.sign(data_json.as_bytes());

    let signed_license = SignedLicense {
        data,
        signature: hex::encode(signature.to_bytes()),
    };

    // 6. Output JSON and Short Key
    let license_json = serde_json::to_string_pretty(&signed_license)?;

    println!("\n--- Generated License JSON ---");
    println!("{}", license_json);
    println!("-------------------------------");

    // 7. Output Short Key (for mock service)
    println!("\n--- Generated Short Product Key (Dev/Mock Only) ---");
    // Hardcoded keys matching MockLicensingService
    const MOCK_PRIVATE_KEY: [u8; 32] = [
        0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43,
        0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65,
        0x43, 0x21,
    ];

    let tier_code = match tier {
        LicenseTier::Premium => "P",
        LicenseTier::Free => "F",
    };
    let mach_part = if machine_id.len() > 6 {
        &machine_id[..6]
    } else {
        &machine_id
    };

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    tier.hash(&mut hasher);
    machine_id.hash(&mut hasher);
    MOCK_PRIVATE_KEY.hash(&mut hasher);
    let hash = format!("{:x}", hasher.finish());
    let short_key = format!("{}-{}-{}", tier_code, mach_part, &hash[..6]).to_uppercase();

    println!("{}", short_key);
    println!("---------------------------------------------------");

    println!("\nTo activate the app:");
    println!("1. Run the app in development mode.");
    println!("2. Go to settings/licensing.");
    println!("3. Paste either the JSON or the Short Key above.");

    Ok(())
}

fn get_machine_id() -> String {
    match machine_uid::get() {
        Ok(uid) => uid,
        Err(e) => {
            tracing::warn!(
                "Failed to get machine UID: {}. Falling back to default identifier.",
                e
            );
            "unknown_machine".to_string()
        }
    }
}
