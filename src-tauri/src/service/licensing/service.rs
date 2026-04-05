use crate::contexts::licensing::{AddonError, LicenseTier, LicenseStatus, LicenseVerifier, LicensingAgent, SignedLicense};
use async_trait::async_trait;
use std::sync::Arc;

pub struct LicensingService {
    verifier: LicenseVerifier,
    settings_repo: Arc<dyn crate::repository::SettingsRepository>,
}

impl LicensingService {
    const PUBLIC_KEY: &'static [u8; 32] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/public_key.bin"));
    const LICENSE_SETTING_KEY: &str = "signed_license";

    pub fn new(
        settings_repo: Arc<dyn crate::repository::SettingsRepository>,
    ) -> Result<Self, AddonError> {
        let verifier = LicenseVerifier::new(Self::PUBLIC_KEY.to_owned())?;
        Ok(Self { verifier, settings_repo })
    }

}

#[async_trait]
impl LicensingAgent for LicensingService {
    async fn load_license(&self) -> Result<LicenseStatus, AddonError> {
        let license_content = self
            .settings_repo
            .get_setting(Self::LICENSE_SETTING_KEY)
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

        self.verifier.verify(&signed_license)
    }

    async fn activate_with_key(&self, key: &str) -> Result<LicenseStatus, AddonError> {
        let signed_license = <Self as LicensingAgent>::decode_key(key)?;

        // Verify signature + determine soft-expiry status.
        let status = self.verifier.verify(&signed_license)?;

        // Persist so the app survives restarts.
        let license_json =
            serde_json::to_string(&signed_license).map_err(|_| AddonError::VerificationFailed)?;

        self.settings_repo
            .set_setting(Self::LICENSE_SETTING_KEY, &license_json)
            .await
            .map_err(|_| AddonError::VerificationFailed)?;

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::licensing::{LicenseData, TierVersion};
    use crate::repository::sqlite_settings_repo;
    use crate::test_utils::fixtures::setup_test_db;
    use base64::Engine;
    use ed25519_dalek::Signer;
    use rand::rngs::OsRng;

    fn make_key(signing_key: &ed25519_dalek::SigningKey, tier: LicenseTier, expires_at: Option<chrono::DateTime<chrono::Utc>>) -> String {
        let data = LicenseData {
            key: "test-order".to_string(),
            machine_id: "*".to_string(),
            tier,
            tier_version: TierVersion::new(1, 0, 0),
            tier_meta: None,
            expires_at,
            issued_at: chrono::Utc::now(),
        };
        let data_json = serde_json::to_string(&data).unwrap();
        let signature = signing_key.sign(data_json.as_bytes());
        let signed = SignedLicense { data, signature: hex::encode(signature.to_bytes()) };
        let json = serde_json::to_string(&signed).unwrap();
        format!("{KEY_PREFIX}{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json))
    }

    async fn setup() -> (LicensingService, ed25519_dalek::SigningKey) {
        let pool = setup_test_db().await;
        let settings_repo = sqlite_settings_repo(pool);
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let mut service = LicensingService::new(settings_repo).unwrap();
        service.verifier = LicenseVerifier::new(signing_key.verifying_key().to_bytes()).unwrap();
        (service, signing_key)
    }

    #[tokio::test]
    async fn load_license_free_when_empty() {
        let (service, _) = setup().await;
        assert_eq!(service.load_license().await.unwrap(), LicenseStatus::Active(LicenseTier::Free));
    }

    #[tokio::test]
    async fn activate_and_reload_premium() {
        let (service, signing_key) = setup().await;
        let key = make_key(&signing_key, LicenseTier::Premium, Some(chrono::Utc::now() + chrono::Duration::days(365)));
        let status = service.activate_with_key(&key).await.unwrap();
        assert_eq!(status, LicenseStatus::Active(LicenseTier::Premium));
        assert_eq!(service.load_license().await.unwrap(), LicenseStatus::Active(LicenseTier::Premium));
    }

    #[tokio::test]
    async fn activate_with_updates_expired_key() {
        let (service, signing_key) = setup().await;
        let key = make_key(&signing_key, LicenseTier::Premium, Some(chrono::Utc::now() - chrono::Duration::days(1)));
        let status = service.activate_with_key(&key).await.unwrap();
        assert_eq!(status, LicenseStatus::UpdatesExpired(LicenseTier::Premium));
    }

    #[tokio::test]
    async fn reject_invalid_key() {
        let (service, _) = setup().await;
        let err = service.activate_with_key("SEOINSIKT-notvalidbase64!!!").await.unwrap_err();
        assert!(matches!(err, AddonError::InvalidLicenseKey));
    }

    #[tokio::test]
    async fn reject_tampered_key() {
        let (service, signing_key) = setup().await;
        let key = make_key(&signing_key, LicenseTier::Free, None);
        // Decode, tamper tier to Premium, re-encode
        let b64 = key.strip_prefix(KEY_PREFIX).unwrap();
        let json_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(b64).unwrap();
        let mut signed: SignedLicense = serde_json::from_slice(&json_bytes).unwrap();
        signed.data.tier = LicenseTier::Premium;
        let tampered_json = serde_json::to_string(&signed).unwrap();
        let tampered_key = format!("{KEY_PREFIX}{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(tampered_json));
        let err = service.activate_with_key(&tampered_key).await.unwrap_err();
        assert!(matches!(err, AddonError::InvalidSignature));
    }
}
