use crate::domain::licensing::{
    AddonError, LicenseData, LicenseVerifier, LicensingAgent, SignedLicense,
};
use crate::domain::permissions::LicenseTier;
use crate::service::hardware::HardwareService;
use async_trait::async_trait;
use chrono::Timelike;
use std::sync::Arc;

pub struct MockLicensingService {
    settings_repo: Arc<dyn crate::repository::SettingsRepository>,
    public_key: [u8; 32],
    private_key: [u8; 32],
}

impl MockLicensingService {
    // Hardcoded keys for development
    const MOCK_PRIVATE_KEY: [u8; 32] = [
        0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43,
        0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65,
        0x43, 0x21,
    ];

    pub fn new(settings_repo: Arc<dyn crate::repository::SettingsRepository>) -> Self {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&Self::MOCK_PRIVATE_KEY);
        let public_key = signing_key.verifying_key().to_bytes();

        Self {
            settings_repo,
            public_key,
            private_key: Self::MOCK_PRIVATE_KEY,
        }
    }

    /// Verifies a short product key format: TIER-MACH-SIG
    /// e.g. P-ABCDEF-GHIJKL
    fn verify_short_key(&self, key: &str) -> Option<LicenseTier> {
        let parts: Vec<&str> = key.split('-').collect();
        if parts.len() != 3 {
            return None;
        }

        let tier = match parts[0] {
            "P" => LicenseTier::Premium,
            "F" => LicenseTier::Free,
            _ => return None,
        };

        let _mach_part = parts[1];
        let sig_part = parts[2];

        // Verify "signature" by hashing (Tier + MachineID + Secret)
        // In a real system this would be more complex, but for mock we use this.
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        tier.hash(&mut hasher);
        HardwareService::get_machine_id().hash(&mut hasher);
        self.private_key.hash(&mut hasher);
        let expected_hash = format!("{:x}", hasher.finish());

        // Match truncated hash (first 6 chars)
        if sig_part.to_lowercase() == expected_hash[..sig_part.len()].to_lowercase() {
            Some(tier)
        } else {
            None
        }
    }

    /// Generates a valid short key for a tier (for testing/utility)
    pub fn generate_short_key(&self, tier: LicenseTier) -> String {
        let tier_code = match tier {
            LicenseTier::Premium => "P",
            LicenseTier::Free => "F",
        };

        // Use a short version of machine ID for the MACH part
        let mach_id = HardwareService::get_machine_id();
        let mach_part = if mach_id.len() > 6 {
            &mach_id[..6]
        } else {
            &mach_id
        };

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        tier.hash(&mut hasher);
        mach_id.hash(&mut hasher);
        self.private_key.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());

        format!("{}-{}-{}", tier_code, mach_part, &hash[..6]).to_uppercase()
    }
}

#[async_trait]
impl LicensingAgent for MockLicensingService {
    async fn load_license(&self) -> Result<LicenseTier, AddonError> {
        let license_content = self
            .settings_repo
            .get_setting("signed_license")
            .await
            .map_err(|_| AddonError::VerificationFailed)?;

        let Some(license_content) = license_content else {
            return Ok(LicenseTier::Free);
        };

        if license_content.is_empty() {
            return Ok(LicenseTier::Free);
        }

        let signed_license: SignedLicense =
            serde_json::from_str(&license_content).map_err(|_| AddonError::InvalidSignature)?;

        let verifier = LicenseVerifier::new(self.public_key)?;
        verifier.verify(&signed_license, &HardwareService::get_machine_id())
    }

    async fn activate_with_key(&self, key: &str) -> Result<LicenseTier, AddonError> {
        use ed25519_dalek::Signer;

        tracing::info!("[MOCK] Activating license with key: {}", key);

        // 1. Try to verify as a short Product Key
        let tier = if let Some(tier) = self.verify_short_key(key) {
            tracing::info!("[MOCK] Short Product Key verified successfully: {:?}", tier);
            tier
        } else {
            tracing::warn!("[MOCK] Invalid product key, defaulting to Free tier.");
            return Ok(LicenseTier::Free);
        };

        let machine_id = HardwareService::get_machine_id();

        // 2. "Redeem" the short key by generating a full digital license
        let data = LicenseData {
            key: key.to_string(),
            machine_id: machine_id.clone(),
            tier,
            tier_version: crate::domain::TierVersion::new(1, 0, 0),
            tier_meta: None,
            expires_at: None,
            issued_at: chrono::Utc::now().with_nanosecond(0).unwrap(),
        };

        let data_json = serde_json::to_string(&data).map_err(|_| AddonError::VerificationFailed)?;
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&self.private_key);
        let signature = signing_key.sign(data_json.as_bytes());

        let signed_license = SignedLicense {
            data,
            signature: hex::encode(signature.to_bytes()),
        };

        let license_json =
            serde_json::to_string(&signed_license).map_err(|_| AddonError::VerificationFailed)?;

        tracing::info!("[MOCK] Redeemed short key for full license.");

        self.settings_repo
            .set_setting("signed_license", &license_json)
            .await
            .map_err(|e| {
                tracing::error!("[MOCK] Failed to save license to repo: {}", e);
                AddonError::VerificationFailed
            })?;

        Ok(tier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite_settings_repo;
    use crate::test_utils::fixtures::setup_test_db;

    #[tokio::test]
    async fn test_mock_activation_fallbacks() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("app=debug")
            .with_test_writer()
            .try_init();
        let pool = setup_test_db().await;
        let settings_repo = sqlite_settings_repo(pool.clone());
        let service = MockLicensingService::new(settings_repo);

        // 1. Generate a valid short key for Premium
        let premium_key = service.generate_short_key(LicenseTier::Premium);
        println!("Generated Premium Key: {}", premium_key);

        // 2. Activate with valid key -> Premium
        let tier = service.activate_with_key(&premium_key).await.unwrap();
        assert_eq!(tier, LicenseTier::Premium);

        // 3. Verify that a full license was saved and can be loaded
        let loaded_tier = service.load_license().await.unwrap();
        assert_eq!(loaded_tier, LicenseTier::Premium);

        // 4. Activate with invalid key -> Free
        let tier = service.activate_with_key("INVALID-KEY").await.unwrap();
        assert_eq!(tier, LicenseTier::Free);

        // 5. Activate with tampered key (wrong tier) -> Free
        let tampered_key = premium_key.replace("P-", "F-");
        let tier = service.activate_with_key(&tampered_key).await.unwrap();
        assert_eq!(tier, LicenseTier::Free);
    }
}
