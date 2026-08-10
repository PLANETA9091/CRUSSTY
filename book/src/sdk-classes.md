# Classes & JNI

In plain words: **find and poke server classes from your module.** The
server loads its code through its own class loaders, so the normal JNI
lookup often can't see them — the SDK can. Two helpers:

- `find_class` — get a class that's already loaded,
- `wait_class` — wait until the server has loaded it (the server is still
  booting while your module initializes!).

## find_class

```rust
classes::find_class("org/bukkit/Bukkit")?;   // by internal name "a/b/C"
classes::find_class_bytes("dev/dist/DistKernel")?; // by JVM byte-name "a.b.C"
```

Resolves a loaded class across class loaders. Internally uses
`GetLoadedClasses` on the runtime's env — this is how modules reach kernel
classes (PluginClassLoader etc.) that a plain `FindClass` from native code
could never see.

- Kernel classes are available after the runtime's attach, but the
  meaningful ones (Bukkit API) only after the clip/boot phase — see
  `wait_class`.

## wait_class

```rust
let klass = classes::wait_class("org/bukkit/Bukkit", Some(60_000))?;
```

Polls `find_class` up to a deadline on a worker thread. Use this instead of
sleep-looping on the main thread: kernel classes appear asynchronously.

## retransform_class

```rust
classes::retransform_class(&patched_bytes)?;
```

Feeds replacement bytes for an **already loaded** class through the runtime's
`retransform_class` ABI call (JVMTI `RetransformClasses`). The class name is
read from the bytes themselves. Requires the kernel to be past its
instrumentation window (post-clip) — retransforming too early fails with a
JVMTI error the runtime logs with both the raw code and `GetErrorName`
(looking up the name can itself fail — the raw code is the ground truth).

## jni_util

- `with_attached(env, f)` — runs `f` with a valid `JNIEnv`, attaching and
  detaching if the current thread was unattached. Never call JNI from the
  runtime's attach thread or hook threads; use `with_attached` on your own
  threads only.
- `cstr(s)` — null-terminated string for JNI `GetStaticMethodID` etc.

## method / static_method

```rust
classes::static_method("dev/dist/DistKernel", "hello", "(Ljava/lang/String;)V")?;
```

Name-based method lookup (no heavy initialization needed); a future SDK
version caches jmethodIDs per class.
