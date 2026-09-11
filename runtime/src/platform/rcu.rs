//! RCU snapshot cell (TASK-164): an atomically swapped `Arc<T>` with a
//! generation counter and a nanosecond-scale reader quiescence window.
//!
//! This is the read-mostly primitive behind the sub-10ns hot paths: registry
//! views (events), router tables (scheduler), packet hooks (network) and the
//! transform rule set all publish immutable snapshots through an [`ArcCell`].
//!
//! # Read paths
//!
//! * `gen()` — one acquire load. `0` means "never stored": callers use it as
//!   the zero-subscription / zero-router default gate.
//! * `load_arc()` — refcounted clone of the current snapshot. The reader
//!   bumps `readers` for the few nanoseconds the load+clone takes, which is
//!   the only window a writer has to wait for before retiring the old
//!   snapshot. Dispatch code keeps the returned `Arc` alive for as long as
//!   it borrows from the view, so writers never block on long handler runs.
//!
//! # Write path
//!
//! `store()` publishes a fresh snapshot (release), bumps the generation
//! (so per-thread caches can invalidate), then waits for the old pointer's
//! reader windows to drain and retires the old `Arc`. Mutations are rare
//! (subscribe / add_router / register), readers are hot — the classic RCU
//! shape, implemented on `std` atomics only.

use std::hint::spin_loop;
use std::sync::atomic::{AtomicPtr, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// FNV-1a 64-bit over u64 chunks (TASK-164): one xor+multiply per 8 bytes
/// for the long class/topic names on the hot paths, byte-wise FNV tail.
/// Collision safety comes from the full string compare that every consumer
/// runs on a hash hit — the hash only feeds open addressing and memo keys.
#[inline]
pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let (chunks, tail) = bytes.as_chunks::<8>();
    for c in chunks {
        let word = u64::from_le_bytes(*c);
        h = (h ^ word).wrapping_mul(0x1_0000_0001_b3);
    }
    for &b in tail {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x1_0000_0001_b3);
    }
    h
}

pub struct ArcCell<T: ?Sized> {
    /// Heap cell (`Box::into_raw`) holding the current `Arc<T>` snapshot;
    /// null until the first store. The indirection keeps unsized snapshots
    /// (`[Hook]` slices) storable in an `AtomicPtr`.
    ptr: AtomicPtr<Arc<T>>,
    /// Bumped on every store; 0 = empty default shape (fast gate).
    gen: AtomicU64,
    /// Readers inside the load+clone window (`load_arc` only — dispatch
    /// holds a refcount instead, so this never tracks long handler runs).
    readers: AtomicUsize,
}

// SAFETY: T is Send + Sync; the cell only hands out Arc<T> clones and raw
// pointer access is confined to the retire window, which readers guard via
// the `readers` counter.
unsafe impl<T: ?Sized + Send + Sync> Send for ArcCell<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for ArcCell<T> {}

impl<T: ?Sized + Send + Sync + 'static> ArcCell<T> {
    pub const fn new() -> Self {
        Self {
            ptr: AtomicPtr::new(std::ptr::null_mut()),
            gen: AtomicU64::new(0),
            readers: AtomicUsize::new(0),
        }
    }

    /// Snapshot generation; one acquire load. Zero = never mutated.
    #[inline]
    pub fn gen(&self) -> u64 {
        self.gen.load(Ordering::Acquire)
    }

    /// Refcounted clone of the current snapshot (`None` before first store).
    pub fn load_arc(&self) -> Option<Arc<T>> {
        self.readers.fetch_add(1, Ordering::Acquire);
        let p = self.ptr.load(Ordering::Acquire);
        let out = if p.is_null() {
            None
        } else {
            // SAFETY: the reader count is held for this window, so the
            // writer cannot retire the cell before our refcount lands. The
            // cell is a `Box<Arc<T>>` — deref gives the live `Arc<T>`.
            Some(unsafe { (*p).clone() })
        };
        self.readers.fetch_sub(1, Ordering::Release);
        out
    }

    /// Publish `next` and retire the previous snapshot. The generation bump
    /// happens after the pointer swap (release), so a reader that observes
    /// the new generation will observe the new snapshot through any acquire
    /// load that follows.
    pub fn store(&self, next: Arc<T>) {
        let newp = Box::into_raw(Box::new(next));
        let oldp = self.ptr.swap(newp, Ordering::AcqRel);
        self.gen.fetch_add(1, Ordering::Release);
        Self::retire(oldp, &self.readers);
    }

    fn retire(oldp: *mut Arc<T>, readers: &AtomicUsize) {
        if oldp.is_null() {
            return;
        }
        let mut spins = 0usize;
        while readers.load(Ordering::Acquire) != 0 {
            spin_loop();
            spins += 1;
            if spins % 4096 == 0 {
                std::thread::yield_now();
            }
        }
        // SAFETY: unique writer ownership; every reader of this cell has
        // drained its window (or holds a refcount on the inner Arc), so
        // dropping the Box — and with it our Arc reference — is last.
        drop(unsafe { Box::from_raw(oldp) });
    }
}

impl<T: ?Sized> Drop for ArcCell<T> {
    fn drop(&mut self) {
        let p = *self.ptr.get_mut();
        if p.is_null() {
            return;
        }
        while self.readers.load(Ordering::Acquire) != 0 {
            std::thread::yield_now();
        }
        // SAFETY: as `retire` — cell teardown is the final writer.
        drop(unsafe { Box::from_raw(p) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_load_gen() {
        let cell: ArcCell<Vec<u32>> = ArcCell::new();
        assert_eq!(cell.gen(), 0);
        assert!(cell.load_arc().is_none());
        cell.store(Arc::new(vec![1, 2, 3]));
        assert_eq!(cell.gen(), 1);
        let v = cell.load_arc().expect("stored");
        assert_eq!(&*v, &[1, 2, 3]);
        cell.store(Arc::new(vec![4]));
        assert_eq!(cell.gen(), 2);
        let v2 = cell.load_arc().expect("stored 2");
        assert_eq!(&*v2, &[4]);
        drop(v); // old snapshot alive until the last reader drops
        assert_eq!(&*v2, &[4]);
    }

    #[test]
    fn concurrent_readers_never_see_freed_memory() {
        let cell: Arc<ArcCell<String>> = Arc::new(ArcCell::new());
        cell.store(Arc::new("first".to_string()));
        let stop = Arc::new(AtomicU64::new(0));
        let mut handles = Vec::new();
        for _ in 0..4 {
            let cell = Arc::clone(&cell);
            let stop = Arc::clone(&stop);
            handles.push(std::thread::spawn(move || {
                let mut seen = 0u64;
                loop {
                    // At least one load per reader: the 2000 tiny stores can
                    // finish before a slow reader thread is first scheduled.
                    if let Some(s) = cell.load_arc() {
                        assert!(s.starts_with("first") || s.starts_with("second"));
                        seen += 1;
                    }
                    if stop.load(Ordering::Relaxed) == 1 {
                        break;
                    }
                }
                seen
            }));
        }
        for i in 0..2000 {
            cell.store(Arc::new(format!("second-{i}")));
        }
        stop.store(1, Ordering::Relaxed);
        for h in handles {
            assert!(h.join().unwrap() > 0);
        }
    }
}
