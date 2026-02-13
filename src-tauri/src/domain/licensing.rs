use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub enum Addon {
    LinkAnalysis,
    UnlimitedPages,
    GraphView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, specta::Type)]
pub enum LicenseTier {
    #[default]
    Free,
    Premium,
}

pub trait Tier {
    fn available_addons(&self) -> HashSet<Addon>;
}

impl Tier for LicenseTier {
    fn available_addons(&self) -> HashSet<Addon> {
        let mut addons = HashSet::new();
        match self {
            LicenseTier::Free => {
                // Free tier might have limited or no addons
            }
            LicenseTier::Premium => {
                addons.insert(Addon::LinkAnalysis);
                addons.insert(Addon::UnlimitedPages);
                addons.insert(Addon::GraphView);
            }
        }
        addons
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub tier: LicenseTier,
    pub active_addons: HashSet<Addon>,
}

impl UserPermissions {
    pub fn default() -> Self {
        Self {
            tier: LicenseTier::Free,
            active_addons: HashSet::new(),
        }
    }

    pub fn new(tier: LicenseTier) -> Self {
        Self {
            tier,
            active_addons: tier.available_addons(),
        }
    }

    pub fn check_addon(&self, addon: Addon) -> bool {
        self.active_addons.contains(&addon)
    }

    pub fn check_addon_str(&self, addon_name: &str) -> bool {
        self.active_addons
            .iter()
            .any(|a| format!("{:?}", a) == addon_name)
    }

    pub fn update_from_tier(&mut self, tier: LicenseTier) {
        self.tier = tier;
        self.active_addons = tier.available_addons();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    pub key: String,
    pub machine_id: String,
    pub tier: LicenseTier,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedLicense {
    pub data: LicenseData,
    pub signature: String, // Hex encoded signature
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct LicenseActivationRequest {
    pub key: String,
    pub machine_id: String,
}

pub struct LicenseVerifier {
    public_key: ed25519_dalek::VerifyingKey,
}

impl LicenseVerifier {
    pub fn new(public_key_bytes: [u8; 32]) -> Result<Self, AddonError> {
        let public_key = ed25519_dalek::VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|_| AddonError::InvalidPublicKey)?;
        Ok(Self { public_key })
    }

    pub fn verify(
        &self,
        signed_license: &SignedLicense,
        current_machine_id: &str,
    ) -> Result<LicenseTier, AddonError> {
        // 1. Verify Hardware Binding
        if signed_license.data.machine_id != current_machine_id {
            return Err(AddonError::HardwareMismatch);
        }

        // 2. Verify Expiration
        if let Some(expiry) = signed_license.data.expires_at {
            if expiry < chrono::Utc::now() {
                return Err(AddonError::LicenseExpired);
            }
        }

        // 3. Verify Signature
        use ed25519_dalek::Verifier;

        // Use JSON for stable serialization across different platforms/languages
        let data_json = serde_json::to_string(&signed_license.data)
            .map_err(|_| AddonError::VerificationFailed)?;

        let signature_bytes =
            hex::decode(&signed_license.signature).map_err(|_| AddonError::InvalidSignature)?;

        let signature = ed25519_dalek::Signature::from_slice(&signature_bytes)
            .map_err(|_| AddonError::InvalidSignature)?;

        self.public_key
            .verify(data_json.as_bytes(), &signature)
            .map_err(|_| AddonError::InvalidSignature)?;

        Ok(signed_license.data.tier)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error, specta::Type)]
pub enum AddonError {
    #[error("Feature {0:?} is locked. Access denied. Please activate your license.")]
    FeatureLocked(Addon),
    #[error("Invalid license signature.")]
    InvalidSignature,
    #[error("License has expired.")]
    LicenseExpired,
    #[error("License is tied to another machine.")]
    HardwareMismatch,
    #[error("Internal verification error.")]
    VerificationFailed,
    #[error("Invalid public key configuration.")]
    InvalidPublicKey,
    #[error("Network error during activation.")]
    NetworkError,
    #[error("Invalid license key.")]
    InvalidLicenseKey,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_tier_no_addons() {
        let perms = UserPermissions::new(LicenseTier::Free);
        assert!(!perms.check_addon(Addon::LinkAnalysis));
        assert!(!perms.check_addon(Addon::UnlimitedPages));
        assert!(!perms.check_addon(Addon::GraphView));
    }

    #[test]
    fn test_premium_tier_all_addons() {
        let perms = UserPermissions::new(LicenseTier::Premium);
        assert!(perms.check_addon(Addon::LinkAnalysis));
        assert!(perms.check_addon(Addon::UnlimitedPages));
        assert!(perms.check_addon(Addon::GraphView));
    }

    #[test]
    fn test_check_addon_str() {
        let perms = UserPermissions::new(LicenseTier::Premium);
        assert!(perms.check_addon_str("LinkAnalysis"));
        assert!(!perms.check_addon_str("NonExistentAddon"));
    }

    #[test]
    fn test_tier_update() {
        let mut perms = UserPermissions::new(LicenseTier::Free);
        assert!(!perms.check_addon(Addon::LinkAnalysis));

        perms.update_from_tier(LicenseTier::Premium);
        assert!(perms.check_addon(Addon::LinkAnalysis));
        assert_eq!(perms.tier, LicenseTier::Premium);
    }

    #[test]
    fn test_license_verification() {
        use ed25519_dalek::Signer;
        use rand::rngs::OsRng;

        // 1. Generate Keypair
        let mut csprng = OsRng;
        let signing_key: ed25519_dalek::SigningKey =
            ed25519_dalek::SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();

        // 2. Create data
        let machine_id = "test-machine-id".to_string();
        let data = LicenseData {
            key: "AAAA-BBBB-CCCC".to_string(),
            machine_id: machine_id.clone(),
            tier: LicenseTier::Premium,
            expires_at: None,
            issued_at: chrono::Utc::now(),
        };

        // 3. Sign data
        let data_json = serde_json::to_string(&data).unwrap();
        let signature = signing_key.sign(data_json.as_bytes());
        let signed_license = SignedLicense {
            data,
            signature: hex::encode(signature.to_bytes()),
        };

        // 4. Verify
        let verifier = LicenseVerifier::new(public_key.to_bytes()).unwrap();
        let result = verifier.verify(&signed_license, &machine_id);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LicenseTier::Premium);
    }
}
