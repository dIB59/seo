use crate::domain::permissions::{LicenseTier, PermissionRequest};
use crate::domain::TierVersion;
use serde::{Deserialize, Serialize};

/// Core license payload that is signed by the licensing server. `tier_version`
/// is now a typed `TierVersion` value object (serialized as a string to keep
/// JSON compact and compatible with existing string-form usage).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    pub key: String,
    pub machine_id: String,
    pub tier: LicenseTier,
    pub tier_version: TierVersion,
    pub tier_meta: Option<serde_json::Value>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
}

impl LicenseData {
    /// Convenience: return tuple if `tier_version` is present.
    pub fn tier_version_tuple(&self) -> (u64, u64, u64) {
        (
            self.tier_version.major,
            self.tier_version.minor,
            self.tier_version.patch,
        )
    }
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
        if signed_license.data.machine_id != current_machine_id {
            return Err(AddonError::HardwareMismatch);
        }

        if signed_license
            .data
            .expires_at
            .is_some_and(|expiry| expiry < chrono::Utc::now())
        {
            return Err(AddonError::LicenseExpired);
        }

        use ed25519_dalek::Verifier;

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
#[async_trait::async_trait]
pub trait LicensingAgent: Send + Sync {
    async fn load_license(&self) -> Result<LicenseTier, AddonError>;
    async fn activate_with_key(&self, key: &str) -> Result<LicenseTier, AddonError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error, specta::Type)]
pub enum AddonError {
    #[error("Permission denied for request: {0:?}. Please upgrade your license.")]
    PermissionDenied(PermissionRequest),
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
    fn test_license_verification() {
        use ed25519_dalek::Signer;
        use rand::rngs::OsRng;

        // 1. Generate Keypair
        let mut csprng = OsRng;
        let signing_key: ed25519_dalek::SigningKey =
            ed25519_dalek::SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();

        // 2. Create data (now includes typed `tier_version` + optional tier_meta)
        let machine_id = "test-machine-id".to_string();
        let data = LicenseData {
            key: "AAAA-BBBB-CCCC".to_string(),
            machine_id: machine_id.clone(),
            tier: LicenseTier::Premium,
            tier_version: TierVersion::new(2, 0, 0),
            tier_meta: None,
            expires_at: None,
            issued_at: chrono::Utc::now(),
        };

        // ensure value-object comparisons work as expected
        let tv = data.tier_version;
        assert_eq!(tv, TierVersion::new(2, 0, 0));
        assert!(tv >= TierVersion::new(1, 9, 0));
        assert!(tv >= TierVersion::new(2, 0, 0));
        assert!(tv < TierVersion::new(2, 1, 0));

        // 3. Sign data
        let data_json = serde_json::to_string(&data).unwrap();
        let signature = signing_key.sign(data_json.as_bytes());
        let signed_license = SignedLicense {
            data: data.clone(),
            signature: hex::encode(signature.to_bytes()),
        };

        // 4. Verify
        let verifier = LicenseVerifier::new(public_key.to_bytes()).unwrap();
        let result = verifier.verify(&signed_license, &machine_id);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LicenseTier::Premium);

        // round-trip: deserializing the signed structure should preserve version
        let serialized = serde_json::to_string(&signed_license).unwrap();
        let deserialized: SignedLicense = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.data.tier_version, TierVersion::new(2, 0, 0));
    }

    #[test]
    fn test_license_verification_hardware_mismatch() {
        use ed25519_dalek::Signer;
        use rand::rngs::OsRng;

        let mut csprng = OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();

        let data = LicenseData {
            key: "KEY".to_string(),
            machine_id: "machine-1".to_string(),
            tier: LicenseTier::Premium,
            tier_version: TierVersion::new(1, 0, 0),
            tier_meta: None,
            expires_at: None,
            issued_at: chrono::Utc::now(),
        };

        let data_json = serde_json::to_string(&data).unwrap();
        let signature = signing_key.sign(data_json.as_bytes());
        let signed_license = SignedLicense {
            data,
            signature: hex::encode(signature.to_bytes()),
        };

        let verifier = LicenseVerifier::new(public_key.to_bytes()).unwrap();
        let result = verifier.verify(&signed_license, "machine-2");

        assert!(matches!(result, Err(AddonError::HardwareMismatch)));
    }

    #[test]
    fn test_license_verification_expired() {
        use ed25519_dalek::Signer;
        use rand::rngs::OsRng;

        let mut csprng = OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();

        let data = LicenseData {
            key: "KEY".to_string(),
            machine_id: "machine-1".to_string(),
            tier: LicenseTier::Premium,
            tier_version: TierVersion::new(1, 0, 0),
            tier_meta: None,
            expires_at: Some(chrono::Utc::now() - chrono::Duration::days(1)),
            issued_at: chrono::Utc::now() - chrono::Duration::days(2),
        };

        let data_json = serde_json::to_string(&data).unwrap();
        let signature = signing_key.sign(data_json.as_bytes());
        let signed_license = SignedLicense {
            data,
            signature: hex::encode(signature.to_bytes()),
        };

        let verifier = LicenseVerifier::new(public_key.to_bytes()).unwrap();
        let result = verifier.verify(&signed_license, "machine-1");

        assert!(matches!(result, Err(AddonError::LicenseExpired)));
    }

    #[test]
    fn test_license_verification_invalid_signature() {
        use ed25519_dalek::Signer;
        use rand::rngs::OsRng;

        let mut csprng = OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();

        let data = LicenseData {
            key: "KEY".to_string(),
            machine_id: "machine-1".to_string(),
            tier: LicenseTier::Premium,
            tier_version: TierVersion::new(1, 0, 0),
            tier_meta: None,
            expires_at: None,
            issued_at: chrono::Utc::now(),
        };

        let data_json = serde_json::to_string(&data).unwrap();
        let signature = signing_key.sign(data_json.as_bytes());
        let mut signed_license = SignedLicense {
            data,
            signature: hex::encode(signature.to_bytes()),
        };

        // Corrupt the signature
        signed_license.signature = "00".repeat(64);

        let verifier = LicenseVerifier::new(public_key.to_bytes()).unwrap();
        let result = verifier.verify(&signed_license, "machine-1");

        assert!(matches!(result, Err(AddonError::InvalidSignature)));
    }

    #[test]
    fn test_license_verification_tampered_data() {
        use ed25519_dalek::Signer;
        use rand::rngs::OsRng;

        let mut csprng = OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();

        let data = LicenseData {
            key: "KEY".to_string(),
            machine_id: "machine-1".to_string(),
            tier: LicenseTier::Free, // Originally Free
            tier_version: TierVersion::new(1, 0, 0),
            tier_meta: None,
            expires_at: None,
            issued_at: chrono::Utc::now(),
        };

        let data_json = serde_json::to_string(&data).unwrap();
        let signature = signing_key.sign(data_json.as_bytes());
        let mut signed_license = SignedLicense {
            data,
            signature: hex::encode(signature.to_bytes()),
        };

        // Tamper with data: change tier to Premium
        signed_license.data.tier = LicenseTier::Premium;

        let verifier = LicenseVerifier::new(public_key.to_bytes()).unwrap();
        let result = verifier.verify(&signed_license, "machine-1");

        assert!(matches!(result, Err(AddonError::InvalidSignature)));
    }

    #[test]
    fn test_license_data_tier_version_tuple() {
        let data = LicenseData {
            key: "KEY".to_string(),
            machine_id: "MID".to_string(),
            tier: LicenseTier::Free,
            tier_version: TierVersion::new(1, 2, 3),
            tier_meta: None,
            expires_at: None,
            issued_at: chrono::Utc::now(),
        };

        assert_eq!(data.tier_version_tuple(), (1, 2, 3));
    }
}
