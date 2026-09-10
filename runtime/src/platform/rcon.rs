//! Brick 11: native RCON — an engine-owned remote console channel that
//! cannot stay dead across a CRaC checkpoint/restore cycle.
//!
//! # Why this brick exists (LAW P6B-29)
//!
//! jdk.crac closes unclaimed `java.net` sockets at the JAVA level during
//! checkpoint; the closed state travels in the image. After restore, the
//! kernel's RCON acceptor thread (`net.minecraft.server.rcon.thread.
//! RconThread.run`) loops `accept()` with NO exit path and NO socket
//! re-creation, so every iteration throws `SocketException: Socket closed`
//! before any syscall — a catch-and-continue exception storm and a dead
//! RCON channel. Root-caused and banked in the CRaC rig (a25, LAW P6B-29);
//! the reflective field-swap remedy served 12/12 probes across two restores
//! in the a27 long-soak.
//!
//! # Two remedies, one watcher
//!
//! [`install`] spawns the `crussty-rcon-watch` thread (a plain Rust thread —
//! never permanently attached to the JVM, so there is no JNI thread state to
//! survive a CRIU freeze/restore). Every 2s it evaluates a pure decision
//! ladder ([`next_action`]):
//!
//! 1. **Stand down** while `enable-rcon=false`, or while the game port
//!    (`server-port`) is not LISTENING — the server is still booting (or
//!    dead), and racing the kernel's own RCON bind would break boot.
//! 2. **Remedy 1 — reflective swap** ([`try_reflect_swap`]): the rig-proven
//!    field-swap, ported engine-native. Via JNI (kernel classes are found
//!    loader-agnostically through JVMTI `GetLoadedClasses`; the kernel is
//!    Mojang-mapped, verified by `javap` on the shipped jar): find the live
//!    server instance via `getServer()`, walk `rconThread` → `socket`
//!    fields with reflection, swap in a fresh bound `java.net.ServerSocket`
//!    (reuseAddress, backlog 50). The acceptor bytecode re-reads the field
//!    every iteration, so the storm stops at the swap and the kernel's own
//!    RCON protocol stack serves again. Idempotent: skipped when the port is
//!    already LISTENING.
//! 3. **Remedy 2 — native takeover** ([`start_native_lane`]): if the swap
//!    path cannot run and the port stays dead, CRUSSTY serves RCON itself,
//!    in pure Rust: Source RCON framing, `SERVERDATA_AUTH` (constant-time
//!    password compare), `SERVERDATA_EXECCOMMAND` (dispatched into the
//!    kernel on the console-input queue), `SERVERDATA_RESPONSE_VALUE` /
//!    `SERVERDATA_AUTH_RESPONSE`. The listener is a raw fd owned by Rust —
//!    jdk.crac's Java-level socket close (LAW P6B-29) targets `java.net`
//!    objects only, so the native lane structurally survives restores. Per
//!    command, the dispatch thread attaches to the JVM, queues the command,
//!    and detaches again (no permanent attachment, CRIU-safe).
//!
//! # Command dispatch semantics
//!
//! Commands enter the kernel through the same channel the physical console
//! uses (`handleConsoleInput` — the stdin-fifo lane that TASK-59 fixed on
//! this kernel), with a fallback to
//! `getCommands().performPrefixedCommand(createCommandSourceStack(), cmd)`.
//! Queued commands execute on the server thread next tick. The native lane
//! answers `EXECCOMMAND` with an empty (protocol-legal) `RESPONSE_VALUE`:
//! output capture would require kernel class generation, which the engine
//! does not do.
//!
//! # Hygiene (docs/RCON_HYGIENE_DECISION.md)
//!
//! The password is resolved from `server.properties` `rcon.password` (what
//! the kernel itself reads) falling back to `~/.rcon_password` (the 600-mode
//! file outside every repo). It is compared in constant time and NEVER
//! logged, echoed, or included in any marker. Commands are never logged
//! either — only their byte length.
//!
//! # Markers
//!
//! `eprintln!` markers use the rig's `[crussty-rcon]` + `RCON-*` vocabulary
//! so rig-era log tooling keeps working: `RCON-WATCH`, `RCON-REPAIR-*`,
//! `RCON-NATIVE-*`. An `rcon.native_serving` telemetry metric is published
//! on serving-state changes.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use jni::objects::{JClass, JObject, JString};
use jni::{Env, JavaVM, JValue, errors, jni_sig, jni_str};

use super::publish_metric;

const TAG: &str = "[crussty-rcon]";
const WATCH_INTERVAL: Duration = Duration::from_secs(2);
const BACKLOG: i32 = 50; // a25-proven bind backlog
/// Source RCON payload cap (vanilla `MAX_PAYLOAD_LENGTH`).
const MAX_PAYLOAD: usize = 4096;
/// Packet size cap = id(4) + type(4) + payload(4096) + 2 NULs.
const MAX_PACKET: usize = 4 + 4 + MAX_PAYLOAD + 2;
/// Concurrent native-RCON client cap (vanilla serves one; we allow a few).
const MAX_NATIVE_CLIENTS: usize = 8;
/// Remedy ladder timing (in watch cycles): swap at 2 cycles (~4s), takeover
/// at 6 cycles (~12s) — enough grace for an in-flight swap to bind.
const SWAP_AFTER: u32 = 2;
const TAKEOVER_AFTER: u32 = 6;

fn marker(msg: &str) {
    eprintln!("{TAG} {msg}");
}

// ---------------------------------------------------------------------------
// Configuration (server.properties; password never logged)
// ---------------------------------------------------------------------------

/// Resolved RCON configuration. The password is deliberately never
/// Debug/Display-formatted anywhere; it only ever feeds the constant-time
/// compare.
pub struct RconConfig {
    /// `enable-rcon` from server.properties (vanilla default: false).
    pub enabled: bool,
    /// `rcon.port` (vanilla default 25575).
    pub rcon_port: u16,
    /// `server-port` — the boot gate probe (vanilla default 25565).
    pub game_port: u16,
    /// Resolved password: properties → `~/.rcon_password`. `None` = auth
    /// always fails (fail-closed).
    pub password: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ParsedProperties {
    enable_rcon: Option<bool>,
    rcon_port: Option<u16>,
    server_port: Option<u16>,
    rcon_password: Option<String>,
}

/// Parse server.properties content. Lenient: malformed lines are skipped
/// (same tolerance as the kernel's own loader); later lines win.
fn parse_properties(text: &str) -> ParsedProperties {
    let mut out = ParsedProperties::default();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let Some((k, v)) = t.split_once('=') else {
            continue;
        };
        let (k, v) = (k.trim(), v.trim());
        match k {
            "enable-rcon" => out.enable_rcon = Some(v.eq_ignore_ascii_case("true")),
            "rcon.port" => {
                if let Ok(p) = v.parse() {
                    out.rcon_port = Some(p);
                }
            }
            "server-port" => {
                if let Ok(p) = v.parse() {
                    out.server_port = Some(p);
                }
            }
            "rcon.password" => out.rcon_password = Some(v.to_string()),
            _ => {}
        }
    }
    out
}

/// Read the fallback password file (`~/.rcon_password`, 600-mode, outside
/// every repo — same convention as `~/.git-credentials`).
fn password_from_home_file() -> Option<String> {
    let home = std::env::var_os("HOME")?;
    let path = std::path::Path::new(&home).join(".rcon_password");
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Load the RCON configuration from the working directory's
/// server.properties. A missing file means "not a configured kernel" — the
/// watcher stands down.
pub fn load_config() -> RconConfig {
    let text = std::fs::read_to_string("server.properties").unwrap_or_default();
    let props = parse_properties(&text);
    // No properties file = stand down entirely (vanilla default for
    // enable-rcon is false; no file = no explicit config = no action).
    let have_file = std::path::Path::new("server.properties").is_file();
    RconConfig {
        enabled: have_file && props.enable_rcon.unwrap_or(false),
        rcon_port: props.rcon_port.unwrap_or(25575),
        game_port: props.server_port.unwrap_or(25565),
        password: props
            .rcon_password
            .filter(|s| !s.is_empty())
            .or_else(password_from_home_file),
    }
}

// ---------------------------------------------------------------------------
// Port probe (/proc/net/tcp{,6} — the v12.6 hex-case lesson: ports are
// printed as 4 UPPERCASE hex digits; match the exact field, case-insensitively)
// ---------------------------------------------------------------------------

/// True if a listener is in TCP state `0A` on the given port right now.
pub fn port_listening(port: u16) -> bool {
    let hex = format!("{port:04X}");
    for file in ["/proc/net/tcp", "/proc/net/tcp6"] {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        for line in text.lines().skip(1) {
            let mut fields = line.split_whitespace();
            // fields: sl local_address rem_address st ...
            let Some(local) = fields.nth(1) else { continue };
            let Some(port_field) = local.rsplit(':').next() else { continue };
            // `fields` now sits at rem_address; nth(1) skips it and yields st.
            if port_field.eq_ignore_ascii_case(&hex) && fields.nth(1) == Some("0A") {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Watcher decision ladder (pure — unit-tested)
// ---------------------------------------------------------------------------

/// What the watcher should do this cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    /// Remedy 1: attempt the reflective field swap.
    Swap,
    /// Remedy 2: start the native RCON lane.
    TakeOver,
}

/// The watcher state machine.
///
/// * `rcon_listening` resets the dead-cycle counter (channel healthy — the
///   native lane itself also produces this state once it owns the port).
/// * `enabled == false` → always stand down (counter reset).
/// * game port not listening → the kernel is still booting (or dead); acting
///   here could race the kernel's own RCON bind and break boot. Stand down.
/// * server up + rcon dead for [`SWAP_AFTER`] cycles → [`Action::Swap`];
///   still dead at [`TAKEOVER_AFTER`] cycles → [`Action::TakeOver`].
pub(crate) fn next_action(
    dead_cycles: u32,
    enabled: bool,
    game_listening: bool,
    rcon_listening: bool,
) -> (u32, Option<Action>) {
    if rcon_listening || !enabled || !game_listening {
        return (0, None);
    }
    let cycles = dead_cycles + 1;
    match cycles {
        SWAP_AFTER => (cycles, Some(Action::Swap)),
        TAKEOVER_AFTER => (cycles, Some(Action::TakeOver)),
        _ => (cycles, None),
    }
}

static INSTALLED: AtomicBool = AtomicBool::new(false);

/// Start the RCON repair watcher. Idempotent (a no-op after the first call).
/// Called once from the runtime bring-up; safe to call from any thread.
pub fn install() {
    if INSTALLED.swap(true, Ordering::AcqRel) {
        return;
    }
    match std::thread::Builder::new()
        .name("crussty-rcon-watch".to_string())
        .spawn(watcher_loop)
    {
        Ok(_) => marker("RCON-WATCH installed (2s cadence)"),
        Err(e) => {
            marker(&format!("RCON-WATCH-SPAWN-ERR {e}"));
            INSTALLED.store(false, Ordering::Release);
        }
    }
}

fn watcher_loop() {
    let mut dead_cycles: u32 = 0;
    loop {
        std::thread::sleep(WATCH_INTERVAL);
        let cfg = load_config();
        let game = port_listening(cfg.game_port);
        let rcon = port_listening(cfg.rcon_port);
        let (cycles, action) = next_action(dead_cycles, cfg.enabled, game, rcon);
        dead_cycles = cycles;
        match action {
            None => {}
            Some(Action::Swap) => {
                marker("RCON-REPAIR-ATTEMPT (rcon port dead while server serving)");
                let _ = try_reflect_swap(&cfg);
            }
            Some(Action::TakeOver) => start_native_lane(&cfg),
        }
    }
}

// ---------------------------------------------------------------------------
// JVM handle + exception hygiene
// ---------------------------------------------------------------------------

/// The process-wide jni-rs [`JavaVM`] (same idiom as the side-table brick:
/// built once from the raw `JavaVM*` the agent was handed).
fn jvm() -> Option<&'static JavaVM> {
    static JVM: OnceLock<JavaVM> = OnceLock::new();
    let raw = crate::VM.get().copied().filter(|&p| p != 0)?;
    // Safety: `crate::VM` stores the non-null `JavaVM*` handed to
    // Agent_OnLoad/JNI_OnLoad, valid for the JVM's lifetime.
    Some(JVM.get_or_init(|| unsafe { JavaVM::from_raw(raw as *mut jni::sys::JavaVM) }))
}

/// Clear any pending Java exception (used at every fallible step so a throw
/// never leaks past an attach scope).
fn clear_pending(env: &mut Env<'_>) {
    if env.exception_check() {
        env.exception_clear();
    }
}

// ---------------------------------------------------------------------------
// Kernel class + instance discovery (JVMTI enumeration → direct JNI calls)
// ---------------------------------------------------------------------------

/// Find a loaded class by dotted name via JVMTI `GetLoadedClasses` —
/// loader-agnostic (kernel classes do not live on the system loader, so
/// plain `FindClass` from the agent cannot see them).
/// Returns the class for JNI calls (`call_static_method` takes a `JClass`).
fn find_kernel_class<'local>(env: &mut Env<'local>, dotted: &str) -> Option<JClass<'local>> {
    let jvmti = crate::jvmti_env()?;
    let classes = jvmti.get_loaded_classes().ok()?;
    let want = format!("L{};", dotted.replace('.', "/"));
    for c in classes {
        if let Ok((sig, _generic)) = jvmti.get_class_signature(c) {
            if sig == want {
                // Safety: `c` is a live jclass returned by GetLoadedClasses
                // on this attached thread (jvmti's jclass is `*mut c_void` —
                // same pointer, JNI type).
                return Some(unsafe { JClass::from_raw(env, c as jni::sys::jclass) });
            }
        }
    }
    None
}

/// Find the live server instance via the canonical Mojang-mapped accessor
/// `MinecraftServer.getServer()` (declared on the server class; JNI static
/// resolution walks superclasses, so a lookup from either class name works).
fn find_server_instance<'local>(env: &mut Env<'local>) -> Option<JObject<'local>> {
    for name in [
        "net.minecraft.server.MinecraftServer",
        "net.minecraft.server.dedicated.DedicatedServer",
    ] {
        let Some(class) = find_kernel_class(env, name) else {
            continue;
        };
        match env
            .call_static_method(
                class,
                jni_str!("getServer"),
                jni_sig!("()Lnet/minecraft/server/MinecraftServer;"),
                &[],
            )
            .and_then(|v| v.l())
        {
            Ok(inst) if !inst.is_null() => return Some(inst),
            _ => clear_pending(env),
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Remedy 1 — the rig-proven reflective field swap (engine-native port)
// ---------------------------------------------------------------------------

/// Attempt the reflective `RconThread.socket` swap. Returns `true` when a
/// swap was PERFORMED this call. Idempotent on an already-listening port.
fn try_reflect_swap(cfg: &RconConfig) -> bool {
    let Some(vm) = jvm() else {
        marker("RCON-REPAIR-SKIP no-vm");
        return false;
    };
    match vm.attach_current_thread(|env| reflect_swap(env, cfg)) {
        Ok(v) => v,
        _ => false,
    }
}

/// Walk `name` instance-field declarations up the class hierarchy
/// (`getDeclaredField` → `NoSuchFieldException` → `getSuperclass`).
fn walk_declared_field<'local>(
    env: &mut Env<'local>,
    start_class: JObject<'local>,
    field_name: &str,
) -> Option<JObject<'local>> {
    let mut class = start_class;
    loop {
        let jname = env.new_string(field_name).ok()?;
        match env
            .call_method(
                &class,
                jni_str!("getDeclaredField"),
                jni_sig!("(Ljava/lang/String;)Ljava/lang/reflect/Field;"),
                &[JValue::Object(&jname)],
            )
            .and_then(|v| v.l())
        {
            Ok(field) => return Some(field),
            Err(_) => clear_pending(env), // NoSuchFieldException — expected while walking
        }
        let sup = env
            .call_method(&class, jni_str!("getSuperclass"), jni_sig!("()Ljava/lang/Class;"), &[])
            .ok()?
            .l()
            .ok()?;
        if sup.is_null() {
            return None;
        }
        class = sup;
    }
}

fn make_accessible(env: &mut Env<'_>, member: &JObject<'_>) {
    let _ = env.call_method(member, jni_str!("setAccessible"), jni_sig!("(Z)V"), &[JValue::Bool(true)]);
    clear_pending(env);
}

fn field_type_name(env: &mut Env<'_>, field: &JObject<'_>) -> Option<String> {
    let ty = env
        .call_method(field, jni_str!("getType"), jni_sig!("()Ljava/lang/Class;"), &[])
        .ok()?
        .l()
        .ok()?;
    let name = env
        .call_method(&ty, jni_str!("getName"), jni_sig!("()Ljava/lang/String;"), &[])
        .ok()?
        .l()
        .ok()?;
    let js = unsafe { JString::from_raw(env, name.into_raw()) };
    let s = js.try_to_string(env).ok().map(|s| s.to_string());
    clear_pending(env);
    s
}

fn reflect_swap(env: &mut Env<'_>, cfg: &RconConfig) -> errors::Result<bool> {
    // Double-check idempotence inside the attach scope.
    if port_listening(cfg.rcon_port) {
        marker("RCON-REPAIR-SKIP already-listening");
        return Ok(false);
    }
    let Some(server) = find_server_instance(env) else {
        marker("RCON-REPAIR-SKIP no-instance");
        return Ok(false);
    };
    let server_class = env
        .call_method(&server, jni_str!("getClass"), jni_sig!("()Ljava/lang/Class;"), &[])?
        .l()?;
    // rconThread field (declared on DedicatedServer)
    let Some(rt_field) = walk_declared_field(env, server_class, "rconThread") else {
        marker("RCON-REPAIR-SKIP no-rconThread-field");
        return Ok(false);
    };
    make_accessible(env, &rt_field);
    let rcon_thread = env
        .call_method(
            &rt_field,
            jni_str!("get"),
            jni_sig!("(Ljava/lang/Object;)Ljava/lang/Object;"),
            &[JValue::Object(&server)],
        )?
        .l()?;
    if rcon_thread.is_null() {
        marker("RCON-REPAIR-SKIP rconThread null");
        return Ok(false);
    }
    // socket field — must be declared as java.net.ServerSocket (the type
    // check mirrors the Java repair's `f.getType() != ServerSocket` guard).
    let rt_class = env
        .call_method(&rcon_thread, jni_str!("getClass"), jni_sig!("()Ljava/lang/Class;"), &[])?
        .l()?;
    let mut socket_field: Option<JObject> = None;
    let mut walk = Some(rt_class);
    while let Some(cls) = walk {
        let sup = env
            .call_method(&cls, jni_str!("getSuperclass"), jni_sig!("()Ljava/lang/Class;"), &[])
            .ok()
            .and_then(|v| v.l().ok());
        let Some(field) = walk_declared_field(env, cls, "socket") else {
            clear_pending(env);
            walk = sup.filter(|s| !s.is_null());
            continue;
        };
        make_accessible(env, &field);
        match field_type_name(env, &field) {
            Some(t) if t == "java.net.ServerSocket" => {
                socket_field = Some(field);
                break;
            }
            _ => {
                clear_pending(env);
                walk = sup.filter(|s| !s.is_null());
            }
        }
    }
    let Some(socket_field) = socket_field else {
        marker("RCON-REPAIR-ERR no-socket-field");
        return Ok(false);
    };
    let old = env
        .call_method(
            &socket_field,
            jni_str!("get"),
            jni_sig!("(Ljava/lang/Object;)Ljava/lang/Object;"),
            &[JValue::Object(&rcon_thread)],
        )
        .ok()
        .and_then(|v| v.l().ok());
    let old_state = old
        .as_ref()
        .map(|s| {
            let closed = env
                .call_method(s, jni_str!("isClosed"), jni_sig!("()Z"), &[])
                .ok()
                .and_then(|v| v.z().ok())
                .unwrap_or(false);
            let port = env
                .call_method(s, jni_str!("getLocalPort"), jni_sig!("()I"), &[])
                .ok()
                .and_then(|v| v.i().ok())
                .unwrap_or(-1);
            format!("port={port} closed={closed}")
        })
        .unwrap_or_else(|| "null".to_string());
    marker(&format!(
        "RCON-REPAIR-STAT listen{}={} running=? old{{{old_state}}}",
        cfg.rcon_port,
        port_listening(cfg.rcon_port)
    ));

    // Fresh socket: new ServerSocket(); setReuseAddress(true);
    // bind(new InetSocketAddress(port), 50).
    let socket_class = env.find_class(jni_str!("java/net/ServerSocket"))?;
    let fresh = env.new_object(&socket_class, jni_sig!("()V"), &[])?;
    let _ = env.call_method(&fresh, jni_str!("setReuseAddress"), jni_sig!("(Z)V"), &[JValue::Bool(true)]);
    clear_pending(env);
    let addr_class = env.find_class(jni_str!("java/net/InetSocketAddress"))?;
    let addr = env.new_object(&addr_class, jni_sig!("(I)V"), &[JValue::Int(cfg.rcon_port as i32)])?;
    env.call_method(
        &fresh,
        jni_str!("bind"),
        jni_sig!("(Ljava/net/SocketAddress;I)V"),
        &[JValue::Object(&addr), JValue::Int(BACKLOG)],
    )?;
    // Private final instance field, non-record ⇒ settable (a25-proven live).
    env.call_method(
        &socket_field,
        jni_str!("set"),
        jni_sig!("(Ljava/lang/Object;Ljava/lang/Object;)V"),
        &[JValue::Object(&rcon_thread), JValue::Object(&fresh)],
    )?;
    let bound_port = env
        .call_method(&fresh, jni_str!("getLocalPort"), jni_sig!("()I"), &[])
        .ok()
        .and_then(|v| v.i().ok())
        .unwrap_or(-1);
    marker(&format!(
        "RCON-REPAIR-SWAPPED old{{{old_state}}} -> fresh port={bound_port}"
    ));
    Ok(true)
}

// ---------------------------------------------------------------------------
// Remedy 2 — the native RCON lane (pure Rust protocol server)
// ---------------------------------------------------------------------------

static NATIVE_SERVING: AtomicBool = AtomicBool::new(false);
static NATIVE_CLIENTS: AtomicUsize = AtomicUsize::new(0);

/// Source RCON packet types.
const TYPE_RESPONSE: i32 = 0;
const TYPE_AUTH_RESPONSE: i32 = 2;
const TYPE_EXECCOMMAND: i32 = 2;
const TYPE_AUTH: i32 = 3;

fn publish_serving(on: bool) {
    if NATIVE_SERVING.swap(on, Ordering::AcqRel) != on {
        publish_metric("rcon.native_serving", if on { 1.0 } else { 0.0 }, Some("bool"), None);
        marker(if on {
            "RCON-NATIVE-SERVING"
        } else {
            "RCON-NATIVE-DOWN"
        });
    }
}

/// Remedy 2: bind the RCON port natively and serve the Source RCON protocol
/// from Rust. Called only when the game port is up and the kernel's own
/// listener has been dead for `TAKEOVER_AFTER` cycles, so the bind cannot
/// race a healthy kernel listener.
fn start_native_lane(cfg: &RconConfig) {
    let Some(password) = cfg.password.clone() else {
        marker("RCON-NATIVE-SKIP no-password-source (fail-closed)");
        return;
    };
    match TcpListener::bind(("0.0.0.0", cfg.rcon_port)) {
        Ok(listener) => {
            marker(&format!(
                "RCON-NATIVE-BIND port={} dispatch=console-queue",
                cfg.rcon_port
            ));
            publish_serving(true);
            let pass: Arc<str> = Arc::from(password);
            if let Err(e) = std::thread::Builder::new()
                .name("crussty-rcon-native".to_string())
                .spawn(move || serve_loop(listener, pass))
            {
                marker(&format!("RCON-NATIVE-SPAWN-ERR {e}"));
                publish_serving(false);
            }
        }
        Err(e) => {
            // Someone bound it between probe and bind — re-arm the watcher.
            marker(&format!("RCON-NATIVE-BIND-ERR {e} (re-arming watcher)"));
            publish_serving(false);
        }
    }
}

fn serve_loop(listener: TcpListener, password: Arc<str>) {
    loop {
        match listener.accept() {
            Ok((stream, _addr)) => {
                if NATIVE_CLIENTS.load(Ordering::Relaxed) >= MAX_NATIVE_CLIENTS {
                    drop(stream);
                    continue;
                }
                NATIVE_CLIENTS.fetch_add(1, Ordering::Relaxed);
                let pass = password.clone();
                let spawned = std::thread::Builder::new()
                    .name("crussty-rcon-client".to_string())
                    .spawn(move || {
                        let _ = handle_client(stream, pass);
                        NATIVE_CLIENTS.fetch_sub(1, Ordering::Relaxed);
                    });
                if spawned.is_err() {
                    NATIVE_CLIENTS.fetch_sub(1, Ordering::Relaxed);
                }
            }
            Err(_) => {
                publish_serving(false);
                return; // listener dead — watcher re-arms on the next dead cycles
            }
        }
    }
}

/// Constant-time byte-slice equality (length differences return early — the
/// length is not secret).
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[derive(Debug, PartialEq, Eq)]
struct RconPacket {
    id: i32,
    ptype: i32,
    payload: Vec<u8>,
}

fn read_u32_le(r: &mut impl Read) -> std::io::Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

/// Read one Source RCON packet. `Ok(None)` = clean EOF (client closed).
/// Protocol violations (oversized, short, missing NUL terminators) are
/// `Err` → the connection is closed.
fn read_packet(r: &mut impl Read) -> std::io::Result<Option<RconPacket>> {
    let size = match read_u32_le(r) {
        Ok(s) => s as usize,
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };
    if size == 0 {
        return Ok(None);
    }
    if size < 10 || size > MAX_PACKET {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "rcon: packet size out of range",
        ));
    }
    let mut body = vec![0u8; size];
    r.read_exact(&mut body)?;
    let id = i32::from_le_bytes(body[0..4].try_into().expect("4 bytes"));
    let ptype = i32::from_le_bytes(body[4..8].try_into().expect("4 bytes"));
    let payload = body[8..size - 2].to_vec();
    if body[size - 2] != 0 || body[size - 1] != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "rcon: missing NUL terminators",
        ));
    }
    Ok(Some(RconPacket { id, ptype, payload }))
}

fn write_packet(w: &mut impl Write, id: i32, ptype: i32, payload: &[u8]) -> std::io::Result<()> {
    let size = 4 + 4 + payload.len() + 2;
    let mut buf = Vec::with_capacity(4 + size);
    buf.extend_from_slice(&(size as u32).to_le_bytes());
    buf.extend_from_slice(&id.to_le_bytes());
    buf.extend_from_slice(&ptype.to_le_bytes());
    buf.extend_from_slice(payload);
    buf.extend_from_slice(&[0, 0]);
    w.write_all(&buf)?;
    w.flush()
}

fn handle_client(mut stream: TcpStream, password: Arc<str>) -> std::io::Result<()> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    let mut authed = false;
    loop {
        let Some(pkt) = read_packet(&mut stream)? else {
            return Ok(());
        };
        match pkt.ptype {
            TYPE_AUTH => {
                if ct_eq(&pkt.payload, password.as_bytes()) {
                    authed = true;
                    write_packet(&mut stream, pkt.id, TYPE_AUTH_RESPONSE, b"")?;
                } else {
                    // Protocol: auth failure answers with id = -1, then the
                    // connection is closed (vanilla behavior).
                    write_packet(&mut stream, -1, TYPE_AUTH_RESPONSE, b"")?;
                    return Ok(());
                }
            }
            TYPE_EXECCOMMAND => {
                if !authed {
                    write_packet(&mut stream, pkt.id, TYPE_RESPONSE, b"")?;
                    return Ok(());
                }
                let len = pkt.payload.len();
                let cmd = String::from_utf8_lossy(&pkt.payload).trim().to_string();
                if !cmd.is_empty() {
                    dispatch_command(&cmd);
                }
                // Protocol-legal empty response (module docs: output capture
                // needs kernel class generation, which we do not do).
                marker(&format!("RCON-NATIVE-EXEC len={len}"));
                write_packet(&mut stream, pkt.id, TYPE_RESPONSE, b"")?;
            }
            _ => return Ok(()), // unknown type — drop the connection
        }
    }
}

// ---------------------------------------------------------------------------
// Command dispatch into the kernel (direct JNI ladder, per-op attach)
// ---------------------------------------------------------------------------

/// Queue a console command into the kernel. Never blocks the server thread;
/// the kernel executes queued commands on its own tick.
fn dispatch_command(cmd: &str) {
    let Some(vm) = jvm() else {
        marker("RCON-DISPATCH-SKIP no-vm");
        return;
    };
    let cmd = cmd.to_string();
    let served = matches!(vm.attach_current_thread(|env| dispatch_impl(env, &cmd)), Ok(true));
    if !served {
        static DISPATCH_FAILED: AtomicBool = AtomicBool::new(false);
        if !DISPATCH_FAILED.swap(true, Ordering::AcqRel) {
            marker("RCON-DISPATCH-UNAVAILABLE (both ladders failed)");
        }
    }
}

fn dispatch_impl(env: &mut Env<'_>, cmd: &str) -> errors::Result<bool> {
    let Some(server) = find_server_instance(env) else {
        return Ok(false);
    };
    let jcmd = env.new_string(cmd)?;

    // Ladder 1: DedicatedServer.handleConsoleInput(String) — the physical
    // console queue (the TASK-59-fixed stdin lane lands here). Public;
    // resolution walks the class hierarchy.
    match env.call_method(
        &server,
        jni_str!("handleConsoleInput"),
        jni_sig!("(Ljava/lang/String;)V"),
        &[JValue::Object(&jcmd)],
    ) {
        Ok(_) => return Ok(true),
        Err(_) => clear_pending(env),
    }

    // Ladder 2: server.getCommands().performPrefixedCommand(
    //              server.createCommandSourceStack(), cmd)
    let dispatcher = match env
        .call_method(&server, jni_str!("getCommands"), jni_sig!("()Lnet/minecraft/commands/Commands;"), &[])
        .and_then(|v| v.l())
    {
        Ok(d) => d,
        Err(_) => {
            clear_pending(env);
            return Ok(false);
        }
    };
    let source = match env
        .call_method(&server, jni_str!("createCommandSourceStack"), jni_sig!("()Lnet/minecraft/commands/CommandSourceStack;"), &[])
        .and_then(|v| v.l())
    {
        Ok(s) => s,
        Err(_) => {
            clear_pending(env);
            return Ok(false);
        }
    };
    match env.call_method(
        &dispatcher,
        jni_str!("performPrefixedCommand"),
        jni_sig!("(Lnet/minecraft/commands/CommandSourceStack;Ljava/lang/String;)V"),
        &[JValue::Object(&source), JValue::Object(&jcmd)],
    ) {
        Ok(_) => Ok(true),
        Err(_) => {
            clear_pending(env);
            Ok(false)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests (no JVM — every JVM path no-ops when the VM is absent)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // ---- decision ladder ---------------------------------------------------

    #[test]
    fn decision_ladder() {
        // healthy channel resets cycles
        assert_eq!(next_action(3, true, true, true), (0, None));
        // disabled → stand down
        assert_eq!(next_action(3, false, true, false), (0, None));
        // game port down (booting/dead) → stand down
        assert_eq!(next_action(3, true, false, false), (0, None));
        // server up, rcon dead: cycle 1 = wait
        assert_eq!(next_action(0, true, true, false), (1, None));
        // cycle 2 = swap
        assert_eq!(next_action(1, true, true, false), (2, Some(Action::Swap)));
        // cycle 3..5 = wait (the swap had its grace window)
        assert_eq!(next_action(2, true, true, false), (3, None));
        assert_eq!(next_action(4, true, true, false), (5, None));
        // cycle 6 = takeover
        assert_eq!(next_action(5, true, true, false), (6, Some(Action::TakeOver)));
        // past takeover: nothing new (lane start is once per episode)
        assert_eq!(next_action(6, true, true, false), (7, None));
    }

    // ---- config --------------------------------------------------------------

    #[test]
    fn properties_parse() {
        let text = "# comment\n\
                    enable-rcon=true\n\
                    rcon.port=25600\n\
                    server-port=25566\n\
                    rcon.password= hunter2 \n\
                    motd=whatever\n\
                    enable-rcon=false\n"; // last wins, like the kernel loader
        let p = parse_properties(text);
        assert_eq!(p.enable_rcon, Some(false));
        assert_eq!(p.rcon_port, Some(25600));
        assert_eq!(p.server_port, Some(25566));
        assert_eq!(p.rcon_password.as_deref(), Some("hunter2"));
    }

    #[test]
    fn config_defaults_and_disabled_without_file() {
        let dir = std::env::temp_dir().join(format!("crussty-rcon-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        // no file → stand down entirely
        let cfg = load_config();
        assert!(!cfg.enabled);
        // file with rcon on → enabled, defaults filled
        std::fs::write(dir.join("server.properties"), "enable-rcon=true\nrcon.port=25601\n").unwrap();
        let cfg = load_config();
        assert!(cfg.enabled);
        assert_eq!(cfg.rcon_port, 25601);
        assert_eq!(cfg.game_port, 25565);
        std::env::set_current_dir(prev).unwrap();
        std::fs::remove_dir_all(&dir).ok();
    }

    // ---- port probe ------------------------------------------------------------

    #[test]
    fn port_probe_matches_real_listener() {
        let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = l.local_addr().unwrap().port();
        assert!(port_listening(port), "bound listener must be seen in /proc");
        drop(l);
        // after close the port must be gone (retry briefly for teardown)
        let mut gone = false;
        for _ in 0..50 {
            if !port_listening(port) {
                gone = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(gone, "closed listener must disappear from /proc");
        assert!(!port_listening(1), "no listener on port 1");
    }

    // ---- protocol codec ----------------------------------------------------------

    #[test]
    fn packet_roundtrip() {
        let mut buf = Vec::new();
        write_packet(&mut buf, 7, TYPE_RESPONSE, b"hello").unwrap();
        let pkt = read_packet(&mut Cursor::new(buf)).unwrap().unwrap();
        assert_eq!(pkt.id, 7);
        assert_eq!(pkt.ptype, TYPE_RESPONSE);
        assert_eq!(pkt.payload, b"hello");
    }

    #[test]
    fn packet_empty_payload_roundtrip() {
        let mut buf = Vec::new();
        write_packet(&mut buf, -1, TYPE_AUTH_RESPONSE, b"").unwrap();
        let pkt = read_packet(&mut Cursor::new(buf)).unwrap().unwrap();
        assert_eq!(pkt.id, -1);
        assert_eq!(pkt.ptype, TYPE_AUTH_RESPONSE);
        assert!(pkt.payload.is_empty());
    }

    #[test]
    fn packet_rejects_oversize() {
        let big = 4 + 4 + MAX_PAYLOAD + 1 + 2;
        let mut buf = Vec::new();
        buf.extend_from_slice(&(big as u32).to_le_bytes());
        buf.resize(4 + big, 0);
        assert!(read_packet(&mut Cursor::new(buf)).is_err());
    }

    #[test]
    fn packet_rejects_missing_nuls() {
        let mut body = Vec::new();
        body.extend_from_slice(&1i32.to_le_bytes());
        body.extend_from_slice(&TYPE_AUTH.to_le_bytes());
        body.extend_from_slice(b"pw");
        body.push(0); // only one terminator → invalid
        let mut buf = Vec::new();
        buf.extend_from_slice(&(body.len() as u32).to_le_bytes());
        buf.extend_from_slice(&body);
        assert!(read_packet(&mut Cursor::new(buf)).is_err());
    }

    #[test]
    fn packet_eof_is_none() {
        assert!(read_packet(&mut Cursor::new(Vec::new())).unwrap().is_none());
    }

    // ---- auth ----------------------------------------------------------------------

    #[test]
    fn constant_time_compare() {
        assert!(ct_eq(b"abc", b"abc"));
        assert!(!ct_eq(b"abc", b"abd"));
        assert!(!ct_eq(b"abc", b"abcd"));
        assert!(ct_eq(b"", b""));
    }
}
