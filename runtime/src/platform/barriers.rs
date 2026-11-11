//! Brick 4a: phase barriers — coordinated ticking across module threads.
//!
//! A `PhaseBarrier` lets N threads (typically platform threads created by the
//! `threads` brick) run in lockstep phases: each phase begins when all
//! participants have reached it. Built on `Mutex + Condvar` (std only, no
//! extra dependencies, portable to Windows) because `std::sync::Barrier`
//! cannot express timeouts, dynamic participants, or resets.
//!
//! # State machine
//!
//! One `Mutex` guards the whole barrier state and a single `Condvar` fans out
//! events (`notify_all` on completion/reset/removal). The state is:
//!
//! - `participants`: how many arrivals a phase needs;
//! - `arrived`: how many waiters are parked in the current generation;
//! - `generation`: monotonic phase counter, incremented on every completed or
//!   aborted phase — this is the "completed phase" index returned to waiters;
//! - `terminated_gen`: the generation aborted by [`PhaseBarrier::reset`]
//!   (`None` otherwise), so waiters of that generation can distinguish a
//!   reset from a normal completion even if more phases complete right after.
//!
//! `wait`/`try_wait` record the caller's generation on arrival, then loop on
//! `Condvar::wait_while`/`wait_timeout_while` (the timeout is a *total* budget
//! — `wait_timeout_while` adjusts the remaining time per wakeup) until the
//! generation advances. The first waiter that observes `arrived >=
//! participants` completes the phase: `generation += 1`, `arrived = 0`,
//! `notify_all`; every waiter returns the generation it completed.
//!
//! # Semantics
//!
//! - A waiter that times out leaves `arrived` counted — it is no longer part
//!   of the phase and does not block completion; the phase simply needs one
//!   fewer arrival. This is what makes timeouts useful: a slow or dead
//!   participant can never wedge a phase forever.
//! - [`PhaseBarrier::add_participant`] only affects future phases; waiters
//!   already parked are not re-quoted.
//! - [`PhaseBarrier::remove_participant`] lowers the requirement immediately:
//!   if the remaining arrivals already satisfy it (e.g. a *waiting*
//!   participant was removed), the phase completes right now and all waiters
//!   (including the removed one) return `Ok` — removing a waiting participant
//!   completes its phase, so nobody deadlocks.
//! - [`PhaseBarrier::reset`] aborts the current generation: `try_wait`
//!   waiters of it return `Err(TryWaitError::Terminated)` (and `wait` returns
//!   the aborted phase index — `wait` has no error channel by design, see
//!   below); the barrier is fully reusable for the next generation.
//! - Mutex poisoning is recovered from (`into_inner`), so a panicking
//!   participant cannot wedge the barrier.
//!
//! # Why `wait` returns `usize`, not `Result`
//!
//! The pre-existing API contract (`PhaseBarrier::wait -> usize`) is kept:
//! callers that block indefinitely get the phase index either way, and code
//! that needs to distinguish "reset aborted my phase" from "phase completed"
//! uses [`PhaseBarrier::try_wait`] with a timeout.

use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryWaitError {
    /// The timeout elapsed before all participants arrived.
    Timeout,
    /// The phase was aborted by `reset()` before it completed.
    Terminated,
}

#[derive(Clone)]
pub struct PhaseBarrier {
    state: Arc<Mutex<State>>,
    cv: Arc<Condvar>,
}

struct State {
    participants: usize,
    arrived: usize,
    generation: u64,
    terminated_gen: Option<u64>,
}

/// Internal wait result: every waiter needs the phase index it completed (or
/// was aborted in), so `Terminated` carries it too.
enum WaitResult {
    Phase(u64),
    Timeout,
    Terminated(u64),
}

impl PhaseBarrier {
    pub fn new(participants: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                participants,
                arrived: 0,
                generation: 0,
                terminated_gen: None,
            })),
            cv: Arc::new(Condvar::new()),
        }
    }

    /// Wait for all participants of this phase. Returns the completed phase
    /// index (0-based). Blocks until everyone arrived. If the phase is
    /// aborted by [`reset`](Self::reset), returns the aborted phase index;
    /// use [`try_wait`](Self::try_wait) if you need to distinguish the two.
    pub fn wait(&self) -> usize {
        match self.wait_inner(None) {
            WaitResult::Phase(p) | WaitResult::Terminated(p) => p as usize,
            WaitResult::Timeout => unreachable!("wait() has no timeout"),
        }
    }

    /// Wait at most `timeout` for all participants of this phase. Returns
    /// `Ok(phase)` with the completed phase index, or `Err(Timeout)` /
    /// `Err(Terminated)`. A timed-out waiter leaves its arrival counted: the
    /// phase completes once the remaining participants arrive.
    pub fn try_wait(&self, timeout: Duration) -> Result<usize, TryWaitError> {
        match self.wait_inner(Some(timeout)) {
            WaitResult::Phase(p) => Ok(p as usize),
            WaitResult::Timeout => Err(TryWaitError::Timeout),
            WaitResult::Terminated(_) => Err(TryWaitError::Terminated),
        }
    }

    /// How many arrivals the current phase needs. Use
    /// [`add_participant`](Self::add_participant) /
    /// [`remove_participant`](Self::remove_participant) to change it.
    pub fn participants(&self) -> usize {
        self.lock_state().participants
    }

    /// Add one participant. Affects future phases: waiters already parked in
    /// the current phase still complete it without the new participant.
    pub fn add_participant(&self) {
        let mut s = self.lock_state();
        s.participants += 1;
    }

    /// Remove one participant (no-op if there are none). If the remaining
    /// arrivals already satisfy the requirement — e.g. a *waiting* participant
    /// was removed — the phase completes right now and all its waiters (the
    /// removed one included) return `Ok`, so nobody deadlocks.
    pub fn remove_participant(&self) {
        let mut s = self.lock_state();
        if s.participants == 0 {
            return;
        }
        s.participants -= 1;
        // If the remaining participants are already waiting (a waiting
        // participant was removed, or un-arrived ones dropped out), complete
        // the phase so nobody blocks forever.
        if s.arrived >= s.participants {
            s.generation += 1;
            s.arrived = 0;
            self.cv.notify_all();
        }
    }

    /// Abort the current phase: every waiter of it gets
    /// `Err(TryWaitError::Terminated)` from `try_wait` (or the aborted phase
    /// index from `wait`). The barrier is fully reusable: arrivals after the
    /// reset start a fresh phase.
    pub fn reset(&self) {
        let mut s = self.lock_state();
        s.terminated_gen = Some(s.generation);
        s.generation += 1;
        s.arrived = 0;
        self.cv.notify_all();
    }

    fn wait_inner(&self, timeout: Option<Duration>) -> WaitResult {
        let mut s = self.lock_state();
        if s.participants == 0 {
            // No participants: a phase completes instantly for every caller.
            s.generation += 1;
            return WaitResult::Phase(s.generation - 1);
        }
        let my_gen = s.generation;
        s.arrived += 1;
        loop {
            if s.arrived >= s.participants {
                // We are the last arrival (or a removal made the phase
                // complete): trip the barrier and wake everyone.
                s.generation += 1;
                s.arrived = 0;
                self.cv.notify_all();
                return WaitResult::Phase(my_gen);
            }
            let keep_waiting = |st: &mut State| st.generation == my_gen && st.terminated_gen != Some(my_gen);
            match timeout {
                Some(d) => {
                    let (guard, tr) = self
                        .cv
                        .wait_timeout_while(s, d, keep_waiting)
                        .unwrap_or_else(|p| p.into_inner());
                    s = guard;
                    if s.terminated_gen == Some(my_gen) {
                        return WaitResult::Terminated(my_gen);
                    }
                    if s.generation != my_gen {
                        return WaitResult::Phase(my_gen);
                    }
                    if tr.timed_out() {
                        return WaitResult::Timeout;
                    }
                    // Spurious wakeup: loop and wait again.
                }
                None => {
                    s = self
                        .cv
                        .wait_while(s, keep_waiting)
                        .unwrap_or_else(|p| p.into_inner());
                    if s.terminated_gen == Some(my_gen) {
                        return WaitResult::Terminated(my_gen);
                    }
                    if s.generation != my_gen {
                        return WaitResult::Phase(my_gen);
                    }
                    // Spurious wakeup: loop.
                }
            }
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, State> {
        // Recover from poisoning: a panicking participant must not wedge the
        // barrier (the state it left is still coherent — all mutations happen
        // atomically under the lock).
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn phases_complete_together() {
        let b = PhaseBarrier::new(2);
        let b2 = b.clone();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let t1 = std::thread::spawn(move || {
            for _ in 0..3 {
                b.wait();
                c.fetch_add(1, Ordering::SeqCst);
            }
        });
        let c2 = Arc::clone(&counter);
        let t2 = std::thread::spawn(move || {
            for _ in 0..3 {
                b2.wait();
                c2.fetch_add(1, Ordering::SeqCst);
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();
        // 6 increments total, all after their phase waited
        assert_eq!(counter.load(Ordering::SeqCst), 6);
    }

    #[test]
    fn phase_indices_advance_together() {
        let b = PhaseBarrier::new(2);
        let b2 = b.clone();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let s1 = Arc::clone(&seen);
        let t1 = std::thread::spawn(move || {
            for _ in 0..3 {
                let p = b.wait();
                s1.lock().unwrap().push(p);
            }
        });
        let s2 = Arc::clone(&seen);
        let t2 = std::thread::spawn(move || {
            for _ in 0..3 {
                let p = b2.wait();
                s2.lock().unwrap().push(p);
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();
        let mut v = seen.lock().unwrap().clone();
        v.sort_unstable();
        assert_eq!(v, vec![0, 0, 1, 1, 2, 2]);
    }

    #[test]
    fn try_wait_times_out() {
        let b = PhaseBarrier::new(2);
        let start = std::time::Instant::now();
        let r = b.try_wait(Duration::from_millis(50));
        assert_eq!(r, Err(TryWaitError::Timeout));
        assert!(start.elapsed() >= Duration::from_millis(30), "must actually wait");
    }

    #[test]
    fn try_wait_ok_when_phase_completes() {
        let b = PhaseBarrier::new(2);
        let b2 = b.clone();
        let t1 = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            b.wait();
        });
        let r = b2.try_wait(Duration::from_secs(1));
        t1.join().unwrap();
        assert_eq!(r, Ok(0));
    }

    #[test]
    fn timed_out_waiter_does_not_block_completion() {
        let b = PhaseBarrier::new(2);
        let r = b.try_wait(Duration::from_millis(30));
        assert_eq!(r, Err(TryWaitError::Timeout));
        // The timed-out caller's arrival is still counted: one fresh arrival
        // completes the phase (documented "leaves arrived counted" semantics).
        assert_eq!(b.try_wait(Duration::from_millis(100)), Ok(0));
    }

    #[test]
    fn reset_terminates_waiting_threads() {
        let b = PhaseBarrier::new(2);
        let b2 = b.clone();
        let done = Arc::new(AtomicBool::new(false));
        let d = Arc::clone(&done);
        let parked = Arc::new(AtomicBool::new(false));
        let p = Arc::clone(&parked);
        let t = std::thread::spawn(move || {
            p.store(true, Ordering::SeqCst);
            let r = b2.try_wait(Duration::from_secs(5));
            d.store(true, Ordering::SeqCst);
            assert_eq!(r, Err(TryWaitError::Terminated));
        });
        wait_until_parked(&parked);
        b.reset();
        t.join().unwrap();
        assert!(done.load(Ordering::SeqCst));
    }

    #[test]
    fn usable_after_reset() {
        let b = PhaseBarrier::new(2);
        let b2 = b.clone();
        let parked = Arc::new(AtomicBool::new(false));
        let p = Arc::clone(&parked);
        let result = Arc::new(Mutex::new(None));
        let r = Arc::clone(&result);
        let t = std::thread::spawn(move || {
            p.store(true, Ordering::SeqCst);
            *r.lock().unwrap() = Some(b2.try_wait(Duration::from_secs(5)));
        });
        wait_until_parked(&parked);
        b.reset();
        t.join().unwrap();
        assert_eq!(*result.lock().unwrap(), Some(Err(TryWaitError::Terminated)));
        // Next phase: both sides wait again and complete normally.
        let b3 = b.clone();
        let t2 = std::thread::spawn(move || {
            b3.wait();
        });
        let p = b.wait();
        t2.join().unwrap();
        assert_eq!(p, 1);
    }

    #[test]
    fn removing_waiting_participant_wakes_everyone() {
        let b = PhaseBarrier::new(3);
        let b2 = b.clone();
        let b3 = b.clone();
        let woken = Arc::new(AtomicUsize::new(0));
        let w1 = Arc::clone(&woken);
        let w2 = Arc::clone(&woken);
        let parked = Arc::new(AtomicBool::new(false));
        let p = Arc::clone(&parked);
        let t1 = std::thread::spawn(move || {
            p.store(true, Ordering::SeqCst);
            let _ = b.wait();
            w1.fetch_add(1, Ordering::SeqCst);
        });
        let t2 = std::thread::spawn(move || {
            let _ = b2.wait();
            w2.fetch_add(1, Ordering::SeqCst);
        });
        wait_until_parked(&parked);
        // Removes the third, still-waiting participant: the phase completes
        // immediately and everyone (including the removed waiter) returns.
        b3.remove_participant();
        t1.join().unwrap();
        t2.join().unwrap();
        assert_eq!(woken.load(Ordering::SeqCst), 2);
        assert_eq!(b3.participants(), 2);
    }

    #[test]
    fn adding_participant_mid_phase_delays_completion() {
        let b = PhaseBarrier::new(2);
        let b2 = b.clone();
        let done = Arc::new(AtomicBool::new(false));
        let d = Arc::clone(&done);
        let parked = Arc::new(AtomicBool::new(false));
        let p = Arc::clone(&parked);
        let t1 = std::thread::spawn(move || {
            p.store(true, Ordering::SeqCst);
            let _ = b.wait();
            d.store(true, Ordering::SeqCst);
        });
        wait_until_parked(&parked);
        b2.add_participant(); // now needs 3
        assert_eq!(b2.participants(), 3);
        // With only two potential arrivals the phase must not complete.
        let r = b2.try_wait(Duration::from_millis(80));
        assert_eq!(r, Err(TryWaitError::Timeout));
        assert!(!done.load(Ordering::SeqCst));
        // A third arrival completes it.
        let b3 = b2.clone();
        let t2 = std::thread::spawn(move || {
            let _ = b3.wait();
        });
        t1.join().unwrap();
        t2.join().unwrap();
        assert!(done.load(Ordering::SeqCst));
    }

    /// Spin until `flag` is set (the waiter is about to park), then give it a
    /// slice of CPU time to actually reach the condvar wait — the gap between
    /// flag and `arrived += 1` is nanoseconds, far below the sleep.
    fn wait_until_parked(flag: &AtomicBool) {
        while !flag.load(Ordering::SeqCst) {
            std::hint::spin_loop();
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    #[test]
    fn zero_participants_completes_immediately() {
        let b = PhaseBarrier::new(0);
        assert_eq!(b.wait(), 0);
        assert_eq!(b.wait(), 1);
    }

    #[test]
    fn single_participant_never_blocks() {
        let b = PhaseBarrier::new(1);
        assert_eq!(b.try_wait(Duration::ZERO), Ok(0));
        assert_eq!(b.try_wait(Duration::ZERO), Ok(1));
    }
}
