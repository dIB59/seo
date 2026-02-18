use crate::domain::licensing::{AddonError, LicenseVerifier, LicensingAgent, SignedLicense};
use crate::domain::permissions::LicenseTier;
use crate::service::hardware::HardwareService;
use crate::service::spider::SpiderAgent;
use async_trait::async_trait;
use std::sync::Arc;

pub struct LicensingService {
    verifier: LicenseVerifier,
    settings_repo: Arc<dyn crate::repository::SettingsRepository>,
    spider: Arc<dyn SpiderAgent>,
}

impl LicensingService {
    const PUBLIC_KEY: &'static [u8; 32] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/public_key.bin"));
    const API_BASE_URL: &str = "https://api.graviplex.com/licensing";
    const LICENSE_SETTING_KEY: &str = "signed_license";

    pub fn new(
        settings_repo: Arc<dyn crate::repository::SettingsRepository>,
        spider: Arc<dyn SpiderAgent>,
    ) -> Result<Self, AddonError> {
        let verifier = LicenseVerifier::new(Self::PUBLIC_KEY.to_owned())?;
        Ok(Self {
            verifier,
            settings_repo,
            spider,
        })
    }
}

#[async_trait]
impl LicensingAgent for LicensingService {
    /// Loads the license from the database.
    async fn load_license(&self) -> Result<LicenseTier, AddonError> {
        let license_content = self
            .settings_repo
            .get_setting(Self::LICENSE_SETTING_KEY)
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

        let machine_id = HardwareService::get_machine_id();
        self.verifier.verify(&signed_license, &machine_id)
    }

    /// Activates a license using a key by communicating with the REST API.
    async fn activate_with_key(&self, key: &str) -> Result<LicenseTier, AddonError> {
        let machine_id = HardwareService::get_machine_id();

        let payload = serde_json::to_value(&crate::domain::licensing::LicenseActivationRequest {
            key: key.to_string(),
            machine_id: machine_id.clone(),
        })
        .map_err(|_| AddonError::VerificationFailed)?;

        let response = self
            .spider
            .post_json(&format!("{}/activate", Self::API_BASE_URL), &payload)
            .await
            .map_err(|_| AddonError::NetworkError)?;

        if response.status != 200 {
            return Err(AddonError::InvalidLicenseKey);
        }

        let signed_license: SignedLicense =
            serde_json::from_str(&response.body).map_err(|_| AddonError::NetworkError)?;

        // Verify the received license
        let tier = self.verifier.verify(&signed_license, &machine_id)?;

        // Save valid license to database
        let license_json =
            serde_json::to_string(&signed_license).map_err(|_| AddonError::VerificationFailed)?;

        self.settings_repo
            .set_setting(Self::LICENSE_SETTING_KEY, &license_json)
            .await
            .map_err(|_| AddonError::VerificationFailed)?;

        Ok(tier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TierVersion;
    use crate::repository::sqlite_settings_repo;
    use crate::service::spider::{MockSpider, SpiderResponse};
    use crate::test_utils::fixtures::setup_test_db;
    use ed25519_dalek::Signer;
    use rand::rngs::OsRng;

    async fn setup() -> (LicensingService, Arc<MockSpider>, [u8; 32]) {
        let pool = setup_test_db().await;
        let settings_repo = sqlite_settings_repo(pool);
        let mock_spider = Arc::new(MockSpider {
            html_response: String::new(),
            generic_response: SpiderResponse {
                status: 200,
                body: String::new(),
                url: String::new(),
            },
        });

        let service = LicensingService::new(settings_repo, mock_spider.clone()).unwrap();

        let mut csprng = OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();

        // Injecting the public key into the verifier of the service
        let mut service = service;
        service.verifier = LicenseVerifier::new(public_key.to_bytes()).unwrap();

        (service, mock_spider, signing_key.to_bytes())
    }

    #[tokio::test]
    async fn test_load_license_free_tier_when_empty() {
        let (service, _, _) = setup().await;
        let tier = service.load_license().await.unwrap();
        assert_eq!(tier, LicenseTier::Free);
    }

    #[tokio::test]
    async fn test_load_license_success() {
        let (service, _, priv_key_bytes) = setup().await;
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&priv_key_bytes);

        let data = crate::domain::licensing::LicenseData {
            key: "TEST-KEY".to_string(),
            machine_id: HardwareService::get_machine_id(),
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
        let license_json = serde_json::to_string(&signed_license).unwrap();

        service
            .settings_repo
            .set_setting(LicensingService::LICENSE_SETTING_KEY, &license_json)
            .await
            .unwrap();

        let tier = service.load_license().await.unwrap();
        assert_eq!(tier, LicenseTier::Premium);
    }

    #[tokio::test]
    async fn test_activate_with_key_success() {
        let (service, _, priv_key_bytes) = setup().await;
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&priv_key_bytes);

        let data = crate::domain::licensing::LicenseData {
            key: "ACTIVATE-KEY".to_string(),
            machine_id: HardwareService::get_machine_id(),
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
        let license_json = serde_json::to_string(&signed_license).unwrap();

        struct LocalMockSpider {
            response: SpiderResponse,
        }

        #[async_trait]
        impl SpiderAgent for LocalMockSpider {
            async fn fetch_html(&self, _: &str) -> anyhow::Result<String> {
                Ok(String::new())
            }
            async fn get(&self, _: &str) -> anyhow::Result<SpiderResponse> {
                Ok(self.response.clone())
            }
            async fn post_json(
                &self,
                _: &str,
                _: &serde_json::Value,
            ) -> anyhow::Result<SpiderResponse> {
                Ok(self.response.clone())
            }
        }

        let mock_spider = Arc::new(LocalMockSpider {
            response: SpiderResponse {
                status: 200,
                body: license_json,
                url: "test".to_string(),
            },
        });

        let mut service =
            LicensingService::new(service.settings_repo.clone(), mock_spider).unwrap();
        service.verifier = LicenseVerifier::new(signing_key.verifying_key().to_bytes()).unwrap();

        let tier = service.activate_with_key("ACTIVATE-KEY").await.unwrap();
        assert_eq!(tier, LicenseTier::Premium);

        let saved = service
            .settings_repo
            .get_setting(LicensingService::LICENSE_SETTING_KEY)
            .await
            .unwrap()
            .unwrap();
        let saved_license: SignedLicense = serde_json::from_str(&saved).unwrap();
        assert_eq!(saved_license.data.key, "ACTIVATE-KEY");
    }

    #[tokio::test]
    async fn test_activate_with_key_api_failure() {
        let pool = setup_test_db().await;
        let settings_repo = sqlite_settings_repo(pool);

        struct FailMockSpider;
        #[async_trait]
        impl SpiderAgent for FailMockSpider {
            async fn fetch_html(&self, _: &str) -> anyhow::Result<String> {
                Ok(String::new())
            }
            async fn get(&self, _: &str) -> anyhow::Result<SpiderResponse> {
                Ok(SpiderResponse {
                    status: 404,
                    body: "Not Found".to_string(),
                    url: "test".to_string(),
                })
            }
            async fn post_json(
                &self,
                _: &str,
                _: &serde_json::Value,
            ) -> anyhow::Result<SpiderResponse> {
                Ok(SpiderResponse {
                    status: 404,
                    body: "Not Found".to_string(),
                    url: "test".to_string(),
                })
            }
        }

        let service = LicensingService::new(settings_repo, Arc::new(FailMockSpider)).unwrap();
        let result = service.activate_with_key("FAIL-KEY").await;
        assert!(matches!(result, Err(AddonError::InvalidLicenseKey)));
    }
}
