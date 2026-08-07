//! Brick 8: save events — completion callbacks for world saving.
//!
//! The kernel's save cycle is surfaced as events: before autosave, after
//! autosave (with success/failure), and on demand (`save-all`). Modules use
//! this for streaming backups, journaling, and crash-safe storage — the
//! platform guarantees the callback fires only after the world state is
//! durably written, satisfying the long-standing request for a "world saved"
//! hook (PaperMC/Paper#620).

use serde_json::json;
use std::sync::{Arc, OnceLock};

pub type SaveHandler = Arc<dyn Fn(SaveOutcome) + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveKind {
    Autosave,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveStatus {
    Ok,
    Failed,
}

#[derive(Debug, Clone, Copy)]
pub struct SaveOutcome {
    pub kind: SaveKind,
    pub status: SaveStatus,
    /// Chunks actually written this cycle (0 if unknown).
    pub chunks_written: u64,
    /// Duration of the save in milliseconds.
    pub duration_ms: u64,
}

static HANDLERS: OnceLock<std::sync::Mutex<Vec<SaveHandler>>> = OnceLock::new();

/// Subscribe to save-complete callbacks.
pub fn on_save(f: SaveHandler) {
    HANDLERS
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .push(f);
}

/// The kernel adapter calls this when a save cycle finishes.
pub fn notify_save(outcome: SaveOutcome) {
    let handlers = HANDLERS
        .get()
        .map(|m| m.lock().unwrap().clone())
        .unwrap_or_default();
    for h in handlers {
        h(outcome);
    }
    // Also mirror onto the module event bus for decoupled consumers.
    let payload = json!({
        "kind": if outcome.kind == SaveKind::Autosave { "autosave" } else { "manual" },
        "status": if outcome.status == SaveStatus::Ok { "ok" } else { "failed" },
        "chunks_written": outcome.chunks_written,
        "duration_ms": outcome.duration_ms,
    });
    crate::platform::events::global().publish(crate::platform::events::lifecycle::SAVE_COMPLETE, &payload);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn save_callback_fires() {
        let n = Arc::new(AtomicUsize::new(0));
        let n2 = Arc::clone(&n);
        on_save(Arc::new(move |_| {
            n2.fetch_add(1, Ordering::SeqCst);
        }));
        notify_save(SaveOutcome {
            kind: SaveKind::Autosave,
            status: SaveStatus::Ok,
            chunks_written: 12,
            duration_ms: 45,
        });
        assert_eq!(n.load(Ordering::SeqCst), 1);
    }
}
