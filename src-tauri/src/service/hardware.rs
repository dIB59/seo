use machine_uid;

pub struct HardwareService;

impl HardwareService {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_machine_id_is_non_empty() {
        let id = HardwareService::get_machine_id();
        assert!(!id.is_empty(), "Machine ID should not be empty");
    }

    #[test]
    fn test_get_machine_id_is_consistent() {
        let id1 = HardwareService::get_machine_id();
        let id2 = HardwareService::get_machine_id();
        assert_eq!(id1, id2, "Machine ID should be consistent across calls");
    }
}
