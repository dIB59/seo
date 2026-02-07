use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct JobCanceler {
    cancel_map: Arc<DashMap<String, Arc<AtomicBool>>>,
}

impl JobCanceler {
    pub fn new() -> Self {
        Self {
            cancel_map: Arc::new(DashMap::with_capacity(10)),
        }
    }

    pub fn get_cancel_flag(&self, job_id: &str) -> Arc<AtomicBool> {
        self.cancel_map
            .entry(job_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone()
    }

    pub fn set_cancelled(&self, job_id: &str) {
        if let Some(flag) = self.cancel_map.get(job_id) {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn is_cancelled(&self, job_id: &str) -> bool {
        self.cancel_map
            .get(job_id)
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
    }
}

impl Default for JobCanceler {
    fn default() -> Self {
        Self::new()
    }
}
