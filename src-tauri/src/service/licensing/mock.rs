use crate::contexts::licensing::{
    AddonError, LicenseData, LicenseTier, LicenseStatus, LicenseVerifier, LicensingAgent, SignedLicense,
};
use async_trait::async_trait;
use std::sync::Arc;

pub struct MockLicensingService {
    settings_repo: Arc<dyn crate::repository::SettingsRepository>,
    public_key: [u8; 32],
    private_key: [u8; 32],
}

impl MockLicensingService {
    pub const MOCK_PRIVATE_KEY: [u8; 32] = [
        0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43,
        0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65,
        0x43, 0x21,
    ];

    pub fn new(settings_repo: Arc<dyn crate::repository::SettingsRepository>) -> Self {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&Self::MOCK_PRIVATE_KEY);
        let public_key = signing_key.verifying_key().to_bytes();
        Self { settings_repo, public_key, private_key: Self::MOCK_PRIVATE_KEY }
    }

    /// Generate a valid offline key for the given tier and expiry.
    /// Used in tests and dev builds.
    pub fn generate_license_key(
        &self,
        tier: LicenseTier,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> String {
        use ed25519_dalek::Signer;
        let data = LicenseData {
            key: format!("mock-{}", uuid::Uuid::new_v4()),
            machine_id: "*".to_string(),
            tier,
            tier_version: crate::contexts::licensing::TierVersion::new(1, 0, 0),
            tier_meta: None,
            expires_at,
            issued_at: chrono::Utc::now(),
        };
        let data_json = serde_json::to_string(&data).unwrap();
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&self.private_key);
        let signature = signing_key.sign(data_json.as_bytes());
        let signed = SignedLicense { data, signature: hex::encode(signature.to_bytes()) };
        Self::encode_key(&signed)
    }
}

#[async_trait]
impl LicensingAgent for MockLicensingService {
    async fn load_license(&self) -> Result<LicenseStatus, AddonError> {
        let license_content = self
            .settings_repo
            .get_setting("signed_license")
            .await
            .map_err(|_| AddonError::VerificationFailed)?;

        let Some(content) = license_content else {
            return Ok(LicenseStatus::Active(LicenseTier::Free));
        };

        if content.is_empty() {
            return Ok(LicenseStatus::Active(LicenseTier::Free));
        }

        let signed_license: SignedLicense =
            serde_json::from_str(&content).map_err(|_| AddonError::InvalidSignature)?;

        let verifier = LicenseVerifier::new(self.public_key)?;
        verifier.verify(&signed_license)
    }

    async fn activate_with_key(&self, key: &str) -> Result<LicenseStatus, AddonError> {
        let signed_license = <Self as LicensingAgent>::decode_key(key)?;
        let verifier = LicenseVerifier::new(self.public_key)?;
        let status = verifier.verify(&signed_license)?;

        let license_json =
            serde_json::to_string(&signed_license).map_err(|_| AddonError::VerificationFailed)?;

        self.settings_repo
            .set_setting("signed_license", &license_json)
            .await
            .map_err(|_| AddonError::VerificationFailed)?;

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::sqlite_settings_repo;
    use crate::test_utils::fixtures::setup_test_db;

    #[tokio::test]
    async fn initial_state_is_free() {
        let pool = setup_test_db().await;
        let service = MockLicensingService::new(sqlite_settings_repo(pool));
        assert_eq!(
            service.load_license().await.unwrap(),
            LicenseStatus::Active(LicenseTier::Free)
        );
    }

    #[tokio::test]
    async fn generate_and_activate_premium() {
        let pool = setup_test_db().await;
        let service = MockLicensingService::new(sqlite_settings_repo(pool));
        let key = service.generate_license_key(
            LicenseTier::Premium,
            Some(chrono::Utc::now() + chrono::Duration::days(365)),
        );
        let status = service.activate_with_key(&key).await.unwrap();
        assert_eq!(status, LicenseStatus::Active(LicenseTier::Premium));
        assert_eq!(service.load_license().await.unwrap(), LicenseStatus::Active(LicenseTier::Premium));
    }

    #[tokio::test]
    async fn updates_expired_key_still_activates() {
        let pool = setup_test_db().await;
        let service = MockLicensingService::new(sqlite_settings_repo(pool));
        // 2 days, not 1: see service.rs::activate_with_updates_expired_key
        // for why — BUILD_DATE is compile-day midnight UTC and a 1-day
        // delta lands later in the BUILD_DATE day for tests run the
        // day after compile.
        let key = service.generate_license_key(
            LicenseTier::Premium,
            Some(chrono::Utc::now() - chrono::Duration::days(2)),
        );
        let status = service.activate_with_key(&key).await.unwrap();
        assert_eq!(status, LicenseStatus::UpdatesExpired(LicenseTier::Premium));
    }

    #[tokio::test]
    async fn invalid_key_rejected() {
        let pool = setup_test_db().await;
        let service = MockLicensingService::new(sqlite_settings_repo(pool));
        let err = service.activate_with_key("SEOINSIKT-garbage").await.unwrap_err();
        assert!(matches!(err, AddonError::InvalidLicenseKey));
    }
}
