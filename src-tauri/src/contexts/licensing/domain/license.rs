use super::entitlement::{LicenseTier, PermissionRequest};
use super::tier::TierVersion;
use base64::Engine;
use serde::{Deserialize, Serialize};

pub const KEY_PREFIX: &str = "SEOINSIKT-";

/// Build date embedded at compile time — used for soft-expiry checks.
const BUILD_DATE: &str = env!("BUILD_DATE");

/// Status of a verified license.
///
/// Soft-expiry model: the app never stops working. `expires_at` only controls
/// whether the installed build is within the user's 1-year update window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseStatus {
    /// Build date ≤ `expires_at` (or no expiry) — full access.
    Active(LicenseTier),
    /// Build date > `expires_at` — app works, renewal banner shown.
    UpdatesExpired(LicenseTier),
}

/// Core license payload signed by the developer's private key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    /// Customer / order reference — used for support, not enforced in app.
    pub key: String,
    /// Always `"*"` in Phase 1 (no machine binding). Reserved for Phase 2.
    pub machine_id: String,
    pub tier: LicenseTier,
    pub tier_version: TierVersion,
    pub tier_meta: Option<serde_json::Value>,
    /// End of the update window (purchase date + 1 year).
    /// `None` means the key never expires.
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
}

impl LicenseData {
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
    /// Hex-encoded Ed25519 signature over the JSON of `data`.
    pub signature: String,
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

    /// Verify the Ed25519 signature and determine soft-expiry status.
    /// Machine binding is intentionally skipped (Phase 1 — no server).
    pub fn verify(&self, signed_license: &SignedLicense) -> Result<LicenseStatus, AddonError> {
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

        // Soft expiry: compare the embedded build date against the license window.
        let updates_expired = signed_license
            .data
            .expires_at
            .map(|expiry| {
                chrono::NaiveDate::parse_from_str(BUILD_DATE, "%Y-%m-%d")
                    .ok()
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|dt| dt.and_utc() > expiry)
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        let tier = signed_license.data.tier;
        if updates_expired {
            Ok(LicenseStatus::UpdatesExpired(tier))
        } else {
            Ok(LicenseStatus::Active(tier))
        }
    }
}

#[async_trait::async_trait]
pub trait LicensingAgent: Send + Sync {
    /// Decode a pasted key string into a `SignedLicense`.
    /// `where Self: Sized` keeps the trait object-safe while preventing
    /// implementations from silently re-implementing the wire format.
    fn decode_key(key: &str) -> Result<SignedLicense, AddonError>
    where
        Self: Sized,
    {
        let b64 = key.trim().strip_prefix(KEY_PREFIX).unwrap_or(key.trim());
        let json_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(b64)
            .map_err(|_| AddonError::InvalidLicenseKey)?;
        serde_json::from_slice::<SignedLicense>(&json_bytes)
            .map_err(|_| AddonError::InvalidLicenseKey)
    }

    /// Encode a `SignedLicense` into the `SEOINSIKT-<base64url>` key string.
    fn encode_key(signed: &SignedLicense) -> String
    where
        Self: Sized,
    {
        let json = serde_json::to_string(signed).expect("SignedLicense is always serializable");
        format!(
            "{KEY_PREFIX}{}",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
        )
    }

    async fn load_license(&self) -> Result<LicenseStatus, AddonError>;
    async fn activate_with_key(&self, key: &str) -> Result<LicenseStatus, AddonError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error, specta::Type)]
pub enum AddonError {
    #[error("Permission denied for request: {0:?}. Please upgrade your license.")]
    PermissionDenied(PermissionRequest),
    #[error("Invalid license signature.")]
    InvalidSignature,
    #[error("License is for a different machine.")]
    HardwareMismatch,
    #[error("Internal verification error.")]
    VerificationFailed,
    #[error("Invalid public key configuration.")]
    InvalidPublicKey,
    #[error("Network error during activation.")]
    NetworkError,
    #[error("Invalid license key format.")]
    InvalidLicenseKey,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signed_license(
        signing_key: &ed25519_dalek::SigningKey,
        tier: LicenseTier,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> SignedLicense {
        use ed25519_dalek::Signer;
        let data = LicenseData {
            key: "TEST-KEY".to_string(),
            machine_id: "*".to_string(),
            tier,
            tier_version: TierVersion::new(1, 0, 0),
            tier_meta: None,
            expires_at,
            issued_at: chrono::Utc::now(),
        };
        let data_json = serde_json::to_string(&data).unwrap();
        let signature = signing_key.sign(data_json.as_bytes());
        SignedLicense {
            data,
            signature: hex::encode(signature.to_bytes()),
        }
    }

    #[test]
    fn test_valid_license_no_expiry() {
        use rand::rngs::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let verifier = LicenseVerifier::new(signing_key.verifying_key().to_bytes()).unwrap();

        let signed = make_signed_license(&signing_key, LicenseTier::Premium, None);
        let status = verifier.verify(&signed).unwrap();
        assert_eq!(status, LicenseStatus::Active(LicenseTier::Premium));
    }

    #[test]
    fn test_updates_expired_when_expiry_in_past() {
        use rand::rngs::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let verifier = LicenseVerifier::new(signing_key.verifying_key().to_bytes()).unwrap();

        let past = chrono::Utc::now() - chrono::Duration::days(365);
        let signed = make_signed_license(&signing_key, LicenseTier::Premium, Some(past));
        // Build date is today, license expired a year ago — updates should be expired.
        let status = verifier.verify(&signed).unwrap();
        assert_eq!(status, LicenseStatus::UpdatesExpired(LicenseTier::Premium));
    }

    #[test]
    fn test_active_when_expiry_in_future() {
        use rand::rngs::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let verifier = LicenseVerifier::new(signing_key.verifying_key().to_bytes()).unwrap();

        let future = chrono::Utc::now() + chrono::Duration::days(365);
        let signed = make_signed_license(&signing_key, LicenseTier::Premium, Some(future));
        let status = verifier.verify(&signed).unwrap();
        assert_eq!(status, LicenseStatus::Active(LicenseTier::Premium));
    }

    #[test]
    fn test_tampered_data_rejected() {
        use rand::rngs::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let verifier = LicenseVerifier::new(signing_key.verifying_key().to_bytes()).unwrap();

        let mut signed = make_signed_license(&signing_key, LicenseTier::Free, None);
        signed.data.tier = LicenseTier::Premium; // tamper
        assert!(matches!(verifier.verify(&signed), Err(AddonError::InvalidSignature)));
    }

    #[test]
    fn test_invalid_signature_rejected() {
        use rand::rngs::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let verifier = LicenseVerifier::new(signing_key.verifying_key().to_bytes()).unwrap();

        let mut signed = make_signed_license(&signing_key, LicenseTier::Premium, None);
        signed.signature = "00".repeat(64);
        assert!(matches!(verifier.verify(&signed), Err(AddonError::InvalidSignature)));
    }

    #[test]
    fn test_tier_version_tuple() {
        let data = LicenseData {
            key: "K".to_string(),
            machine_id: "*".to_string(),
            tier: LicenseTier::Free,
            tier_version: TierVersion::new(1, 2, 3),
            tier_meta: None,
            expires_at: None,
            issued_at: chrono::Utc::now(),
        };
        assert_eq!(data.tier_version_tuple(), (1, 2, 3));
    }
}
