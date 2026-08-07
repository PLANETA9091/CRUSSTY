# Crussty-Dist v2 — design (implemented & verified end-to-end)

Goal: the downloaded artifact is a **launcher**. It boots any Paper-family kernel
the user drops into `versions/`, and **injects Rust modules from `modules/`**
(recursive, no fixed API, no limits — our own dist engine becomes one of them).

## Status

VERIFIED live: `launcher.jar -> child JVM (-agentpath) -> libcrussty_runtime.so ->
dlopen(libhello.so) -> ClassFileLoadHook pipeline -> JNI attach ->
Bukkit.getLogger().info()` — message appears in the server log. This is the
"no API" proof: a native plugin talking to a live Purpur kernel via raw JVM.

## Structure

```
v2/
├── launcher.jar          # what users download — Java control plane
│                          #   scan versions/ -> spawn child JVM with runtime
├── libcrussty_runtime.so      # native injection engine (Rust cdylib) — dlopen()s plugins
├── versions/             # user drops the kernel jar here (e.g. purpur-1.21.10.jar)
├── modules/            # Rust modules — recursive scan, subfolders = free grouping
│   ├── hello/            # e2e proof plugin (source lives here; artifact = .so + cplugin.json)
│   └── dist/             # v1 engine as a plugin — NEXT MILESTONE
├── cplug-abi/            # the ONLY ABI crate
├── runtime/                # the JVMTI runtime crate (crussty-runtime, cdylib)
├── launcher/src/main/java/dev/dist/launcher/Main.java
├── config/
└── logs/                 # server.log (tee) + the kernel's own latest.log
```

## Launcher (Main.java)

- `findKernel`: `versions/*.jar`, sorted by name desc (deterministic).
- `findAgent`: `libcrussty_runtime.so` / `libcrussty_runtime.dylib` / `crussty_runtime.dll` at root.
- spawns child JVM (NOT in-process bootstrap — see Sources): `java
  -agentpath:<abs>/libcrussty_runtime.so=modules=<abs>;versions=<abs>;kernel=<name>
  -Xms512M -Xmx2G -XX:+UseG1GC -Dfile.encoding=UTF-8 -jar versions/<kernel> --nogui`
- `DIST_JAVA_OPTS` env overrides JVM flags; extra args forwarded to the kernel.
- tee child stdout+stderr to `logs/server.log` AND console; forwards launcher
  stdin to the server console; propagates exit code.
- uses the running JDK's `bin/java` (JDK 21+).

## Contract (the ONLY ABI) — cplug-abi crate

`cplugin_init(api: *const CPluginApi, vm: JavaVmPtr, options: *const c_char) -> i32`
must be exported (`#[unsafe(no_mangle)]`). `CPluginApi { version=1, register_class_hook,
jvmti_allocate }` — trampolines into the runtime, so plugins never link the runtime.
A `ClassHookFn(name, data, len, out, out_len)` returns `rc==0` to patch: buffer must
come from `jvmti_allocate`, runtime deallocates after copying (no leaks in chains).

Manifest sidecar `cplugin.json`: `{ "id": ..., "version": ..., "main": ... }`.

## Discovery (directory-bundle convention, grounded in LV2 / npm / VST3 / Mattermost)

A plugin is a **directory containing `cplugin.json`** — same shape as an LV2
bundle (`manifest.ttl`), an npm package (`package.json`), a VST3 bundle
(`Contents/Info.plist`) or a Mattermost plugin (`plugin.json`). The manifest
names the entry library in `main` (path relative to the manifest, like
package.json `main` / LV2 `lv2:binary` / Mattermost `server.executable`);
when `main` is absent it defaults to `lib<id>.so` next to the manifest. A
manifest without `id` falls back to the plugin folder name (Obsidian
convention). Everything else in the folder is private to the plugin —
bundled native deps (e.g. `crussty/native/*.so`) have no manifest and are
never dlopened as plugins. This replaced the earlier `.so`-keyed scan and the
`.nlib` rename hack (e2e27/e2e28 verified).

- recursive scan of `modules/` (Fabric's `mods/` is NOT recursive — issue #81;
  recursion is our differentiator), folder = free grouping.
- skip `*.disabled` (Paper convention) and build dirs (`target/ build/ out/
  node_modules/ .git`) — a crate built inside `modules/` would otherwise be
  discovered twice (its own artifact + target/debug copy).
- deterministic order: relative-path sort, then topological by declared deps,
  cycles/unknown deps -> warn + keep sorted order (unit-tested).
- `dlopen(RTLD_LOCAL)` via libloading — one plugin can't shadow another's exports.

## Injection model (verified)

- **child JVM with `-agentpath`** (Option A). Why not in-process bootstrap:
  in-process URLClassLoader cannot obtain Instrumentation without a startup
  flag; child gives classpath/crash isolation and cheap restarts. Precedent:
  PaperMC/tack (Rust launcher), Paperclip's own re-exec.
- Agent `Agent_OnLoad` runs before ANY class loads (even JDK bootstrap) —
  plugins see the whole class lifecycle, from `jdk/internal/vm/...` up.
- Agent capabilities: can_generate_early_class_hook_or_events, can_retransform_classes
  (OnLoad-only), CANNOT use GetInstrumentation (native JVMTI agents (spec term) do not get one) —
  `ClassFileLoadHook` IS the hot-patch pipeline.
- The kernel (purpur jar) is itself a clip: it extracts `versions/`+`libraries/`
  into its cwd and boots the real server **in the same process** (verified) —
  the runtime survives the clip phase and sees the actual server classes.

## Hot-patch pipeline (user requirement: automatic, available to all plugins)

Every class load runs through the runtime's hook chain (plugins registered via
`register_class_hook`). rc==0 + new bytes -> the runtime feeds them to JVMTI
(chained: plugin A's output feeds plugin B). This is the single mechanism that
will carry BOTH the v1 optimizations (as an automatic patch pipeline) and any
plugin's bytecode work. Bytecode rewriting for Java 21+ (major 65) needs frame
recomputation — ASM via an injected Java helper class (can't live in pure Rust).

## Talking to the kernel from native code (verified patterns)

1. `JNI FindClass` from an attached native thread ONLY sees the system loader —
   `org.bukkit.Bukkit` lives in Paper's own class loader -> ClassNotFoundException.
   **Fix: JVMTI `GetLoadedClasses` + `GetClassSignature`, match the descriptor,
   `NewGlobalRef`, use the global ref from any thread.** (Bootstrap classes like
   `java/util/logging/Logger` resolve fine via FindClass.)
2. `Server.getConsoleSender()` descriptor is
   `()Lorg/bukkit/command/ConsoleCommandSender;` (not CommandSender).
3. Paper's AsyncCatcher REFUSES `dispatchCommand()` from non-main threads
   (by design). Async-safe demo call: `Bukkit.getLogger().info(...)`.
   **For sync work: generate a Runnable via `DefineClass` into Paper's class
   loader, then `Scheduler.scheduleSyncDelayedTask` — TODO in the dist plugin.**
4. Attach: `JavaVM.AttachCurrentThread` (null args); detach after; exceptions
   must be checked+cleared after each call batch.
5. The VM's class-name buffer passed to the hook is not reliably NUL-terminated
   at the name's end (leftover bytes from previous names) — read bounded (128).

## Operational notes (learned live)

- First boot requires `eula.txt` (`eula=true`) or the server exits 0 early
  after printing "You need to agree to the EULA".
- `Files.newBufferedWriter(..., APPEND)` without CREATE dies on first run —
  server.log must be opened with `CREATE, APPEND`.
- Launching the stack with stdin at EOF is fine; but test scripts must not
  pkill by a pattern that matches their own command line (bash -c includes it).
- Crash lessons (all hit live during the dist-module port, e2e29-31):
  - First JNI attach from a fresh thread during VM init SIGSEGVs. Modules must
    sleep ~3s after `cplugin_init` (OnLoad runs before VM is ready) before
    calling anything that attaches — same grace the SDK's `on_kernel_ready`
    uses. (`[dist] schedule_main_loop` sleeps 3s.)
  - Two modules defining the same class in the bootstrap loader =
    `LinkageError: duplicate class definition`. Module SDK copies have private
    statics (RTLD_LOCAL), so the runnable class name must be unique per .so
    (`SdkNativeRunnable<addr-of-own-static>`); weave call sites must use
    `main_thread::runnable_class_name()`.
  - `GetMethodID`/`GetStaticMethodID` take a **jclass**, never a jobject —
    passing the server instance object SIGSEGVs the VM. Resolve the method id
    on the class, then `CallObjectMethod(instance, mid, ...)`.

## Next milestones

1. port `crates/mod-native` + dist-paper logic into `modules/dist` (v1 engine
   as a plugin; "наш форк ядра крассти станет тоже плагином").
   - **DONE (modules/dist)**: full v1 engine port — UDP protocol (Ping/Pong,
     Heartbeat, Commit, LeaseGrant/LeaseRevoked, RegionTransfer), fencing
     tombstones, event queue (`type<<32|region`), 100ms main-thread driver
     (poll events -> forceChunks via `DistKernel` helper, heartbeat 1/s,
     commit every `commit-secs` with oracle-tick sync), load from
     `MinecraftServer.getTickTimesNanos()`, SHA-256 region hashing. Helper
     class defined in the kernel's loader (Bukkit API resolves). Live e2e on
     Purpur 1.21.10: 4 leases granted, commits ticking, oracle sees
     Heartbeat(load,ping)/Commit(hash) (e2e31).
   - **DONE (modules/crussty)**: the Crussty CE native surface now ships as a
     c-plugin — `scripts/gen_crussty_table.py` parses the upstream `Java_*`
     exports into `src/jni_table.rs`; the plugin defines all 98 bridge classes
     (bootstrap loader, exact FQCN the export implies) and RegisterNatives
     all 283 symbols from `native/libpaper_native_*.so` at t+3s (e2e25:
     `98 bridge classes, 283 natives registered, 0 unresolved`, live proofs
     `normalNoise.nativeCheck()=1` + ticket summary). Kernel hot-path wirings
     (area_map batch ops, noise handles, ...) are byte-hooks on top of this
     surface — the plugins that wire them are the remaining work.
2. main-thread Runnable recipe (DefineClass + scheduler) for sync kernel ops.
   - **DONE**: runnable class (DefineClass, Java 8 target, RegisterNatives
     run()/weaveMark()) + `MinecraftServer.execute` delivery; live-weave into
     `org/bukkit/Bukkit` verified via retransform (e2e18). Named per module —
     each module carries its own SDK copy (RTLD_LOCAL, private statics), so
     the bootstrap class it defines is `SdkNativeRunnable<addr-of-own-static>`
     (unique per .so) — otherwise the 2nd module hits `LinkageError: duplicate
     class definition`. Weave call sites must use
     `main_thread::runnable_class_name()`, not a hardcoded FQCN.
3. ASM-based Java helper for retransform-class rewrites (optimizations).
4. Windows: `crussty_runtime.dll` + `-Ddist.root` wiring (DB path already handled).
   - **DONE**: launcher passes `-Ddist.root=<root>` to the child JVM (so the
     module/kernel resolution matches regardless of cwd); `run.bat` boots the
     same launcher.jar; runtime default plugin entry is now platform-derived
     (`std::env::consts::{DLL_PREFIX,DLL_SUFFIX}`: `lib<id>.so` / `lib<id>.dylib`
     / `<id>.dll`), covering Windows cdylib names without a `lib` prefix;
     manifest `main` rejects absolute paths on every OS (POSIX `/`, Windows
     `C:\`, UNC `\\`, and `\..\` escapes); crussty's bundled natives glue
     (`MAIN_LIB`/`CHUNK_LIB`) use the same platform naming; `scripts/build-win.sh`
     cross-compiles runtime + all modules to `x86_64-pc-windows-msvc` and stages
     the `.dll`s next to their manifests.

## Sources

- Fabric Loader docs (launchers, Knot classloader, entrypoints, fabric.mod.json);
  DeepWiki: mod discovery top-level only; FabricMC/fabric-loader#81 recursive mods
- jvmti-bindings 2.2.1 (Rust JVMTI); JVMTI ClassFileLoadHook semantics;
  JEP 451 (dynamic agents still allowed 21-25, warning only);
  JEP 472 (System.load warns — `-agentpath` avoids it)
- Paper: AsyncCatcher main-thread rule; CraftConsoleCommandSender descriptors;
  plugin.yml/.disabled conventions; Paperclip/papercli re-exec; PaperMC/tack
- JNI spec: FindClass loader-context rule; GetLoadedClasses/GetClassSignature;
  DefineClass + loader choice; global vs local refs
