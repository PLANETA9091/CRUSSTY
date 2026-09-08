# Shutdown-crash forensics — hs_err "Signal Dispatcher" family (2026-09-07/08) + fix

Session S7-15 (cron 10:23+08, Job 366450). Root-caused the 4 JVM fatal-error
dumps that accumulated across S7-12/S7-13/S7-14 shutdowns; fix = JVMTI phase
gate on the plugin-hook dispatch (runtime commit alongside this report). All
offsets below resolved against build e830e7b9 (build-id match: deployed
/home/z/server/libcrussty_runtime.so ≡ runtime/target/release at 4f9998b^ —
note the deployed binary was 3.5 min OLDER than 4f9998b, so it carried neither
this fix nor the 4f9998b poison-recovery; both ship with the rebuild).

## 1. The four dumps

| hs_err | time (UTC) | thread | first signal | faulting pc resolves to |
|---|---|---|---|---|
| pid18761 | 09-07 19:55 | Server thread | SIGSEGV @ pc=0x0 | (null call; pre-dates the 21:58 runtime rebuild — older module era, kept for the record) |
| pid12181 | 09-07 23:59 | Signal Dispatcher | SIGSEGV @ pc=0x0 | (null call) |
| pid12838 | 09-08 00:02 | Signal Dispatcher | SIGILL (TRAPNO 0x6 = #UD) | modules/crussty/libcrussty.so +0x23050 |
| pid21175 | 09-08 01:45 | Signal Dispatcher | SIGILL (#UD) | modules/crussty/libcrussty.so +0x26fa7 |

The hs_err header ("R15/stack in libcrussty_runtime.so") is MISLEADING: those
are register/stack-residue pointers. The faulting pcs resolve into the
MODULE's (libcrussty.so) `.rodata`, and the value at RSP (the return address)
in pid21175 resolves to `crussty_runtime::CrusstyRuntime::class_file_load_hook
+0x13fe`. The two SIGILL pcs byte-decode as JNI-TABLE STRING LITERALS:

- pid12838 @+0x23050: `"...aryPaperNativeOreFeatureLoopoldL..."` (jni_table.rs
  OreFeatureLoop method-name strings)
- pid21175 @+0x26fa7: `"...erSkipHashes_newLoopSummaryPaper..."` (jni_table.rs
  RemapperSkipHashes strings)

## 2. Proven causal chain (pid21175, identical shape for 12838/12181)

1. JVM shutdown (SIGTERM fallback path; the launcher fifo does not deliver
   console commands — TASK-57 I1) → the VM begins dying and STILL LOADS
   CLASSES (shutdown-hook classes); those loads happen ON the "Signal
   Dispatcher" thread.
2. JVMTI `ClassFileLoadHook` fires on that thread → `CrusstyRuntime::
   class_file_load_hook` runs.
3. The plugin-hook dispatch executes `call *0x20(%rbp)` on the cloned hooks
   table (objdump of the return site, runtime+0x929c8) — the `entry.func`
   it loaded was a pointer INTO THE MODULE'S .rodata strings (or NULL in
   pid12181) instead of `sdk_dispatch_hook`.
4. Executing string bytes → `#UD` → SIGILL (executing nothing → SIGSEGV@0).
5. The JVM error reporter then faults twice while printing (dying VM),
   producing the truncated hs_err shape; the process dies SIGABRT/SIGILL
   (exit 134 family).

Why the table entry was garbage at that moment is NOT proven — candidate
mechanisms (registration-path type confusion, heap reuse under the cloned
snapshot, a stale generation entry) all require the dying-VM execution
environment to matter, and all are MOOT under the fix: **module code no
longer runs when the phase is not LIVE**.

## 3. Fix — JVMTI phase gate on the plugin-hook dispatch

`class_file_load_hook` now clones the hooks table ONLY when
`GetPhase() == JVMTI_PHASE_LIVE`; otherwise the dispatch list is empty and
the module is never entered from a dying VM (the crashing call site becomes
unreachable there). Honest scope notes:

- The platform transform ENGINE (byte-level, no JNI, no module code) is
  deliberately left unconditional: its degrade path is "class runs
  untransformed", and engine rules on dying-VM loads remain valid.
- A `get_phase()` error also skips dispatch (fail-safe direction).
- Normal-phase behavior is byte-identical: LIVE-phase loads dispatch exactly
  as before; the gate only changes the post-VMDeath window.
- This complements (does not supersede) 4f9998b's poison-recovery on the
  same callback-reachable locks.

## 4. Residual risks / follow-ups

- The micro-mechanism (who wrote a &str pointer into `entry.func`) stays
  open; if it recurs IN LIVE phase it is now a live-phase bug with a much
  better observable profile (no dying-VM noise). CRUSSTY_TRACE_HOOKS prints
  `fn={:#x}` per dispatch — a recurrence can be identified by that line.
- pid18761 (SIGSEGV@0 on the Server thread, 19:55) is a different, older
  family (pre-rebuild module); not investigated further — no occurrences
  since 19:55.
- Deploy note: the running server's runtime .so must be REPLACED for the fix
  to apply (boot-time dlopen); a rebuild+deploy happened alongside this fix
  (S7-15), which also ships 4f9998b's poison-recovery that the deployed
  binary predated by 3.5 minutes.
