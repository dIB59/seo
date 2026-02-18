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
