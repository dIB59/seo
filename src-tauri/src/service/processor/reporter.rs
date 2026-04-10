use tauri::Emitter;

// src/service/progress.rs
use serde::Serialize;

#[derive(Debug, Clone, Serialize, specta::Type, tauri_specta::Event)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ProgressEvent {
    Analysis {
        job_id: String,
        progress: f64,
        pages_analyzed: usize,
        total_pages: usize, // Optional total for more accurate progress
    },
    Discovery {
        job_id: String,
        count: usize,
        total_pages: usize,
    },
}

pub trait ProgressEmitter: Send + Sync {
    fn emit(&self, event: ProgressEvent);
}

/// Tauri-specific progress reporter (production implementation only)
pub struct ProgressReporter<R: tauri::Runtime> {
    app_handle: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> Clone for ProgressReporter<R> {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
        }
    }
}

impl<R: tauri::Runtime> ProgressReporter<R> {
    pub fn new(app_handle: tauri::AppHandle<R>) -> Self {
        Self { app_handle }
    }
}

// Single implementation – emit enum variants directly as serialized payloads
impl<R: tauri::Runtime> ProgressEmitter for ProgressReporter<R> {
    fn emit(&self, event: ProgressEvent) {
        // Route to appropriate channel based on event variant
        tracing::trace!("Emitting progress event: {:?}", event);
        let channel = match event {
            ProgressEvent::Analysis { .. } => "analysis:progress",
            ProgressEvent::Discovery { .. } => "discovery:progress",
        };

        // Serialize and emit the enum variant directly – no intermediate struct needed
        if let Err(e) = self.app_handle.emit(channel, &event) {
            tracing::warn!("Failed to emit progress event '{}': {}", channel, e);
        }
    }
}

#[cfg(test)]
mod tests {
    //! Characterization tests for `ProgressEvent`. The wire format
    //! ships through Tauri to the frontend (`tauri_specta::Event`
    //! derive), so the snake_case tag, field names, and the
    //! Analysis/Discovery variant payload shapes are all pinned by the
    //! generated TS bindings.

    use super::*;

    #[test]
    fn analysis_event_serializes_with_event_tag_and_fields() {
        let event = ProgressEvent::Analysis {
            job_id: "job-1".into(),
            progress: 42.5,
            pages_analyzed: 10,
            total_pages: 25,
        };
        let json = serde_json::to_value(&event).unwrap();
        // The internally-tagged enum uses an "event" field with the
        // snake_case variant name.
        assert_eq!(json["event"], "analysis");
        assert_eq!(json["job_id"], "job-1");
        assert_eq!(json["progress"], 42.5);
        assert_eq!(json["pages_analyzed"], 10);
        assert_eq!(json["total_pages"], 25);
    }

    #[test]
    fn discovery_event_serializes_with_distinct_tag() {
        let event = ProgressEvent::Discovery {
            job_id: "job-2".into(),
            count: 7,
            total_pages: 50,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event"], "discovery");
        assert_eq!(json["job_id"], "job-2");
        assert_eq!(json["count"], 7);
        assert_eq!(json["total_pages"], 50);
    }

    #[test]
    fn analysis_and_discovery_have_distinct_serde_tags() {
        let a = serde_json::to_value(ProgressEvent::Analysis {
            job_id: "j".into(),
            progress: 0.0,
            pages_analyzed: 0,
            total_pages: 0,
        })
        .unwrap();
        let d = serde_json::to_value(ProgressEvent::Discovery {
            job_id: "j".into(),
            count: 0,
            total_pages: 0,
        })
        .unwrap();
        assert_ne!(a["event"], d["event"]);
    }

    #[test]
    fn debug_format_includes_variant_name() {
        let event = ProgressEvent::Analysis {
            job_id: "job-x".into(),
            progress: 1.0,
            pages_analyzed: 1,
            total_pages: 1,
        };
        let dbg = format!("{event:?}");
        assert!(dbg.contains("Analysis"));
        assert!(dbg.contains("job-x"));
    }

    #[test]
    fn clone_produces_equivalent_serialization() {
        let event = ProgressEvent::Discovery {
            job_id: "j".into(),
            count: 5,
            total_pages: 10,
        };
        let copy = event.clone();
        let a = serde_json::to_value(&event).unwrap();
        let b = serde_json::to_value(&copy).unwrap();
        assert_eq!(a, b);
    }

    /// Test mock emitter that captures the events passed in via interior
    /// mutability. Useful for unit tests of code that calls `emit`.
    struct CapturingEmitter {
        events: std::sync::Mutex<Vec<ProgressEvent>>,
    }

    impl CapturingEmitter {
        fn new() -> Self {
            Self {
                events: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl ProgressEmitter for CapturingEmitter {
        fn emit(&self, event: ProgressEvent) {
            self.events.lock().unwrap().push(event);
        }
    }

    #[test]
    fn capturing_emitter_records_emit_calls_in_order() {
        let emitter = CapturingEmitter::new();
        emitter.emit(ProgressEvent::Discovery {
            job_id: "j".into(),
            count: 1,
            total_pages: 10,
        });
        emitter.emit(ProgressEvent::Analysis {
            job_id: "j".into(),
            progress: 50.0,
            pages_analyzed: 5,
            total_pages: 10,
        });
        let events = emitter.events.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], ProgressEvent::Discovery { .. }));
        assert!(matches!(events[1], ProgressEvent::Analysis { .. }));
    }
}
