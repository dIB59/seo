use machine_uid;

pub struct HardwareService;

impl HardwareService {
    /// Returns a unique hardware ID for the current machine.
    /// On macOS, this uses the IOPlatformUUID.
    pub fn get_machine_id() -> String {
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

    /// Returns a hashed version of the machine ID for privacy if needed,
    /// but for license binding the raw ID or a stable hash is fine.
    pub fn get_license_id() -> String {
        Self::get_machine_id()
    }
}
