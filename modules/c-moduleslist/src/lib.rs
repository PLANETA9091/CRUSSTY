//! c-moduleslist — `/modules` command, no plugin involved.
//!
//! Mirrors Bukkit's `/plugins`: lists everything the crussty runtime has
//! loaded, scoped to the native module set. No `org.bukkit.plugin.Plugin`
//! object is ever created — the command is a class we define at runtime
//! (unique per .so generation, so hot-reload can re-register cleanly),
//! whose `execute` is a registered JNI native, and which is handed to the
//! server's CommandMap directly.
//!
//! The listed set is discovered the same way the runtime's scan does it:
//! directories carrying a `cplugin.json` under `modules/` (and `plugins/`)
//! relative to the server's CWD; `id` + `version` are read from the manifest
//! with a tiny parser (no JSON crate needed).

use cplug_abi::{CPluginApi, JavaVmPtr};
use cplug_sdk::classes::{find_class, ClassRef};
use cplug_sdk::jni_util::clear_exception;
use jvmti_bindings::prelude::*;
use std::path::Path;
use std::sync::OnceLock;

/// The single required export (cplug-abi contract).
///
/// # Safety
/// `api` must be a valid CPluginApi owned by the agent, `vm` the live
/// JavaVM pointer, `options` a NUL-terminated string for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cplugin_init(
    api: *const CPluginApi,
    vm: JavaVmPtr,
    _options: *const std::ffi::c_char,
) -> i32 {
    eprintln!("[c-moduleslist] cplugin_init");
    cplug_sdk::init(api, vm);

    // Kernel-ready gate: wait for Bukkit.Command (loaded by the server's own
    // class loading long before the first command), then install on the main
    // thread (define_class + RegisterNatives + CommandMap.register all want
    // the server quiescent, and the main thread guarantees that).
    cplug_sdk::hooks::on_kernel_ready("org.bukkit.Bukkit", || {
        let Some(cmd_cls) = cplug_sdk::classes::wait_class("org/bukkit/command/Command", 120_000)
        else {
            eprintln!("[c-moduleslist] org/bukkit/command/Command never loaded");
            return;
        };
        cplug_sdk::run_on_main_thread(move |env| {
            install_command(env, &cmd_cls);
        });
    });
    0
}

// ---------------------------------------------------------------------------
// The command class (per-module-unique name)
// ---------------------------------------------------------------------------

/// The address of our OWN static distinguishes this .so from every other
/// module's (RTLD_LOCAL => private statics) — same trick as the SDK's
/// runnable class. A hot-reloaded generation gets a time suffix too: the
/// kernel frequently maps the replacement .so at the SAME base address (so
/// the static address repeats) and `define_class` would then hit the
/// duplicate-class-definition guard.
static CMD_NAME: OnceLock<Box<str>> = OnceLock::new();

fn cmd_class_name() -> &'static str {
    CMD_NAME.get_or_init(|| {
        static UNIQUE_SENTINEL: u8 = 0;
        let addr = &UNIQUE_SENTINEL as *const u8 as usize;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        format!("dev/crussty/moduleslist/Cmd{addr:x}{nanos:x}").into_boxed_str()
    })
}

/// Bytecode for:
/// ```java
/// public class Cmd<addr> extends org.bukkit.command.Command {
///     public Cmd<addr>(String name) { super(name); }
///     public native boolean execute(CommandSender sender, String label, String[] args);
/// }
/// ```
/// Java 8 target (major 52): no branches => no StackMapTable required.
fn command_class_bytes(name: &str) -> Vec<u8> {
    let mut c = Vec::new();
    c.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]); // magic
    c.extend_from_slice(&[0, 0]); // minor
    c.extend_from_slice(&[0, 52]); // major 52 (Java 8)
    c.extend_from_slice(&[0, 14]); // cp_count (index 0 is reserved; entries 1..13)

    push_utf8(&mut c, name); // 1 Utf8 <name>
    push_utf8(&mut c, "java/lang/Object"); // 2
    push_utf8(&mut c, "<init>"); // 3
    push_utf8(&mut c, "(Ljava/lang/String;)V"); // 4
    push_utf8(&mut c, "org/bukkit/command/Command"); // 5
    push_utf8(&mut c, "execute"); // 6
    push_utf8(&mut c, "(Lorg/bukkit/command/CommandSender;Ljava/lang/String;[Ljava/lang/String;)Z"); // 7
    push_utf8(&mut c, "Code"); // 8
    push_const(&mut c, 7, &[0, 1]); // 9  Class -> #1
    push_const(&mut c, 7, &[0, 2]); // 10 Class -> #2
    push_const(&mut c, 7, &[0, 5]); // 11 Class -> #5
    push_const(&mut c, 12, &[0, 3, 0, 4]); // 12 NameAndType -> #3 #4
    push_const(&mut c, 10, &[0, 11, 0, 12]); // 13 Methodref -> #11 #12

    // access_flags: public | super
    c.extend_from_slice(&[0, 0x21]);
    // this_class = #9, super_class = #11
    c.extend_from_slice(&[0, 9, 0, 11]);
    // interfaces_count = 0
    c.extend_from_slice(&[0, 0]);
    // fields_count = 0
    c.extend_from_slice(&[0, 0]);

    // methods_count = 2
    c.extend_from_slice(&[0, 2]);
    // method 1: public <init>(Ljava/lang/String;)V with Code
    c.extend_from_slice(&[0, 1, 0, 3, 0, 4]); // access, name#3, desc#4
    c.extend_from_slice(&[0, 1]); // attributes_count
    c.extend_from_slice(&[0, 8]); // attribute name "Code"
    c.extend_from_slice(&[0, 0, 0, 18]); // attribute length
    c.extend_from_slice(&[0, 2]); // max_stack
    c.extend_from_slice(&[0, 2]); // max_locals
    c.extend_from_slice(&[0, 0, 0, 6]); // code_length
                                        // aload_0; aload_1; invokespecial #13; return
    c.extend_from_slice(&[0x2a, 0x2b, 0xb7, 0, 13, 0xb1]);
    c.extend_from_slice(&[0, 0]); // exception_table_length
    c.extend_from_slice(&[0, 0]); // code attributes

    // method 2: public native execute(...)Z  (access 0x0101 = pub|native)
    c.extend_from_slice(&[0x01, 0x01, 0, 6, 0, 7]);
    c.extend_from_slice(&[0, 0]);

    // class attributes_count = 0
    c.extend_from_slice(&[0, 0]);
    c
}

fn push_const(c: &mut Vec<u8>, tag: u8, data: &[u8]) {
    c.push(tag);
    c.extend_from_slice(data);
}

fn push_utf8(c: &mut Vec<u8>, s: &str) {
    c.push(1);
    let b = s.as_bytes();
    c.extend_from_slice(&(b.len() as u16).to_be_bytes());
    c.extend_from_slice(b);
}

// ---------------------------------------------------------------------------
// JNI install: define the class, register the native, register the command
// ---------------------------------------------------------------------------

fn install_command(env: &JniEnv, cmd_cls_ref: &ClassRef) {
    let name = cmd_class_name().to_string();
    let bytes = command_class_bytes(&name);

    // The command class extends org.bukkit.command.Command, so it must be
    // defined into the loader that can resolve it (find_class → JVMTI
    // GetClassLoader; bootstrap classes would answer null, Command is not).
    let loader = {
        let jvmti = Jvmti::new(cplug_sdk::vm() as *mut jni::JavaVM);
        let Some(jvmti) = jvmti.as_ref().ok() else {
            eprintln!("[c-moduleslist] jvmti env unavailable");
            return;
        };
        match jvmti.get_class_loader(cmd_cls_ref.as_jclass()) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[c-moduleslist] get_class_loader(Command) failed: {e:?}");
                return;
            }
        }
    };

    let Some(cls) = env.define_class(&name, loader, &bytes) else {
        eprintln!("[c-moduleslist] define_class({name}) failed");
        if env.exception_check() {
            env.exception_describe();
            env.exception_clear();
        }
        return;
    };

    let natives = [jni::JNINativeMethod {
        name: c"execute".as_ptr(),
        signature: c"(Lorg/bukkit/command/CommandSender;Ljava/lang/String;[Ljava/lang/String;)Z"
            .as_ptr(),
        fnPtr: on_execute as *mut std::ffi::c_void,
    }];
    if env.register_natives(cls, &natives).is_err() {
        eprintln!("[c-moduleslist] register_natives(execute) failed");
        if env.exception_check() {
            env.exception_describe();
            env.exception_clear();
        }
        env.delete_local_ref(cls);
        return;
    }

    // The class was defined into the server's loader, so JNI FindClass (which
    // only sees the calling thread's context loader) cannot resolve it back
    // by name — keep the jclass from define_class for the registration.
    register_with_server(env, cls);
    env.delete_local_ref(cls);
}

/// `server.getCommandMap().register("crussty", new Cmd("modules"))`.
fn register_with_server(env: &JniEnv, cmd_cls: jni::jclass) {
    let clean_up = |env: &JniEnv| {
        if env.exception_check() {
            env.exception_describe();
            env.exception_clear();
        }
    };

    let Some(bukkit) = find_class("org/bukkit/Bukkit") else {
        eprintln!("[c-moduleslist] org/bukkit/Bukkit not loaded");
        return;
    };
    let Some(get_server) =
        env.get_static_method_id(bukkit.as_jclass(), "getServer", "()Lorg/bukkit/Server;")
    else {
        clean_up(env);
        eprintln!("[c-moduleslist] no Bukkit.getServer()");
        return;
    };
    let server = env.call_static_object_method(bukkit.as_jclass(), get_server, &[]);
    if server.is_null() {
        clean_up(env);
        eprintln!("[c-moduleslist] Bukkit.getServer() null");
        return;
    }
    let server_cls = env.get_object_class(server);
    let Some(get_map) = env.get_method_id(
        server_cls,
        "getCommandMap",
        "()Lorg/bukkit/command/CommandMap;",
    ) else {
        clean_up(env);
        eprintln!("[c-moduleslist] no getCommandMap()");
        return;
    };
    let map = env.call_object_method(server, get_map, &[]);
    if map.is_null() {
        clean_up(env);
        eprintln!("[c-moduleslist] getCommandMap() null");
        return;
    }

    let Some(ctor) = env.get_method_id(cmd_cls, "<init>", "(Ljava/lang/String;)V") else {
        clean_up(env);
        eprintln!("[c-moduleslist] no ctor");
        return;
    };
    let Some(label) = env.new_string("modules") else {
        clean_up(env);
        return;
    };
    let Some(cmd) = env.new_object(cmd_cls, ctor, &[jni::jvalue { l: label }]) else {
        clean_up(env);
        eprintln!("[c-moduleslist] new Cmd failed");
        return;
    };
    let map_cls = env.get_object_class(map);
    // `CommandMap.register(String, Command)` returns boolean — the classic
    // old-Bukkit `void register(...)` signature no longer exists on Paper,
    // so a GetMethodID lookup with a `)V` descriptor comes back null while
    // every other inherited method (also boolean-returning ones, e.g.
    // dispatch) resolves fine. The map object is a CraftCommandMap that
    // declares only <init>/getKnownCommands; register is inherited from
    // SimpleCommandMap and must be looked up with its real descriptor.
    let reg_sig = "(Ljava/lang/String;Lorg/bukkit/command/Command;)Z";
    let Some(reg) = env.get_method_id(map_cls, "register", reg_sig) else {
        clean_up(env);
        eprintln!("[c-moduleslist] CommandMap.register ({reg_sig}) not found");
        return;
    };
    let Some(prefix) = env.new_string("crussty") else {
        clean_up(env);
        return;
    };
    let ok = env.call_boolean_method(
        map,
        reg,
        &[jni::jvalue { l: prefix }, jni::jvalue { l: cmd }],
    );
    if env.exception_check() {
        clean_up(env);
        eprintln!("[c-moduleslist] CommandMap.register threw");
        return;
    }
    eprintln!("[c-moduleslist] /modules registered (register returned {ok})");
}

// ---------------------------------------------------------------------------
// Native execute(): reply exactly like /plugins, from the manifest scan
// ---------------------------------------------------------------------------

/// JNI native entry for `Cmd.execute(CommandSender, String, String[])`.
/// Runs on the server's command thread (already attached).
unsafe extern "system" fn on_execute(
    env_raw: *mut jni::JNIEnv,
    _cmd: jni::jobject,
    sender: jni::jobject,
    _label: jni::jstring,
    _args: jni::jobjectArray,
) -> jni::jboolean {
    let env = JniEnv::from_raw(env_raw);
    if sender.is_null() {
        return 1;
    }
    let msg = module_list_message();
    let sender_cls = env.get_object_class(sender);
    if let Some(sm) = env.get_method_id(sender_cls, "sendMessage", "(Ljava/lang/String;)V") {
        if let Some(text) = env.new_string(&msg) {
            env.call_void_method(sender, sm, &[jni::jvalue { l: text }]);
            let _ = clear_exception(&env);
        }
    }
    1
}

fn module_list_message() -> String {
    let mut v = module_snapshot();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    let total = v.len();
    if total == 0 {
        return "Modules (0)".to_string();
    }
    let body = v
        .iter()
        .map(|(id, ver)| match ver {
            Some(v) => format!("{id} v{v}"),
            None => id.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("Modules ({total}): {body}")
}

/// Scan of the module roots (same convention as the runtime's scan): every
/// directory under `modules/` (plus `plugins/`) that carries a
/// `cplugin.json`; build output dirs are skipped so Cargo artifact dirs
/// inside a module never register as modules themselves.
fn module_snapshot() -> Vec<(String, Option<String>)> {
    const BUILD_DIRS: &[&str] = &["target", "build", "out", "node_modules", ".git"];
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut out = Vec::new();
    for root in [cwd.join("modules"), cwd.join("plugins")] {
        walk_manifests(&root, BUILD_DIRS, &mut out);
    }
    out
}

fn walk_manifests(
    dir: &Path,
    build_dirs: &[&str],
    out: &mut Vec<(String, Option<String>)>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let path = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if !path.is_dir() {
            continue;
        }
        if build_dirs.contains(&name.as_str()) {
            continue;
        }
        let manifest = path.join("cplugin.json");
        if manifest.is_file() {
            if let Some(text) = std::fs::read_to_string(&manifest).ok() {
                let id = json_field(&text, "id").unwrap_or_else(|| name.clone());
                let version = json_field(&text, "version");
                out.push((id, version));
            }
        } else {
            walk_manifests(&path, build_dirs, out);
        }
    }
}

/// Minimal JSON string-field reader: finds `"key" : "value"` and returns the
/// (unescaped) value. Good enough for the one-line manifests we own; garbage
/// input just yields None.
fn json_field(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let mut rest = text;
    while let Some(pos) = rest.find(&needle) {
        let after = rest[pos + needle.len()..].trim_start();
        if let Some(value) = after.strip_prefix(':') {
            let value = value.trim_start();
            if let Some(body) = value.strip_prefix('"') {
                let mut out = String::new();
                let mut escaped = false;
                for ch in body.chars() {
                    if escaped {
                        out.push(ch);
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        return Some(out);
                    } else {
                        out.push(ch);
                    }
                }
                return None; // unterminated string
            }
        }
        rest = after;
    }
    None
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_field_reads_id_and_version() {
        let m = r#"{"id":"c-moduleslist","version":"0.1.0"}"#;
        assert_eq!(json_field(m, "id").as_deref(), Some("c-moduleslist"));
        assert_eq!(json_field(m, "version").as_deref(), Some("0.1.0"));
        assert_eq!(json_field(m, "missing"), None);
    }

    #[test]
    fn json_field_handles_escaped_quotes() {
        assert_eq!(
            json_field(r#"{"k":"a \"b\"c"}"#, "k").as_deref(),
            Some("a \"b\"c")
        );
    }

    #[test]
    fn generated_command_class_is_wellformed() {
        let b = command_class_bytes("dev/crussty/moduleslist/Cmd1abc");
        assert_eq!(&b[0..4], &[0xCA, 0xFE, 0xBA, 0xBE]);
        assert_eq!(u16::from_be_bytes([b[8], b[9]]), 14, "cp_count");
    }

    #[test]
    fn snapshot_skips_build_dirs_and_reads_versions() {
        let dir = std::env::temp_dir().join(format!("cmoduleslist-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("modules/hello")).unwrap();
        std::fs::write(dir.join("modules/hello/cplugin.json"), r#"{"id":"hello","version":"0.1.0"}"#)
            .unwrap();
        std::fs::create_dir_all(dir.join("modules/group/nested")).unwrap();
        std::fs::write(
            dir.join("modules/group/cplugin.json"),
            r#"{"id":"grouped"}"#,
        )
        .unwrap();
        std::fs::create_dir_all(dir.join("modules/hello/target/debug")).unwrap();
        std::fs::write(
            dir.join("modules/hello/target/debug/cplugin.json"),
            r#"{"id":"junk","version":"0.0.0"}"#,
        )
        .unwrap();

        // The scanner reads CWD-relative modules/ + plugins/; emulate by
        // pointing CWD at the fixture via the walk directly.
        let mut out = Vec::new();
        walk_manifests(&dir.join("modules"), &["target", "build", "out", "node_modules", ".git"], &mut out);
        out.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(out.len(), 2, "got {out:?}");
        assert_eq!(out[0].0, "grouped");
        assert_eq!(out[0].1, None);
        assert_eq!(out[1].0, "hello");
        assert_eq!(out[1].1.as_deref(), Some("0.1.0"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}