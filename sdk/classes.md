---
title: Classes & JNI
parent: Module SDK
nav_order: 2
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/java.svg" alt=""> Classes & JNI

## find_class

```rust
classes::find_class("org/bukkit/Bukkit")?;   // by internal name "a/b/C"
classes::find_class("dev.dist.DistKernel")?; // dotted names are normalized ("a.b.C" → "a/b/C")
```

Resolves a loaded class across class loaders. Internally uses
`GetLoadedClasses` on the runtime's env — this is how modules reach kernel
classes (PluginClassLoader etc.) that a plain `FindClass` from native code
could never see. Both internal (`/`) and dotted (`.`) forms are normalized
before lookup.

- Kernel classes are available after the runtime's attach, but the
  meaningful ones (Bukkit API) only after the clip/boot phase — see
  `wait_class`.

## wait_class

```rust
let klass = classes::wait_class("org/bukkit/Bukkit", 60_000)?;
```

Polls `find_class` in 200 ms steps until the deadline. Note this blocks
the calling thread — call it from your own threads, never from the main
thread or a hook callback. Kernel classes appear asynchronously.

## retransform

```rust
let ok = classes::retransform("org/bukkit/Bukkit");
```

Re-enters the runtime's `retransform_class` ABI call (JVMTI
`RetransformClasses`) for an **already loaded** class, by internal name —
registered byte hooks see the class again. Returns `bool` (false on any JVMTI
error). Requires the kernel to be past its instrumentation window
(post-clip) — retransforming too early fails with a JVMTI error the runtime
logs with both the raw code and `GetErrorName` (looking up the name can
itself fail — the raw code is the ground truth).

## jni_util

- `with_attached(env, f)` — runs `f` with a valid `JNIEnv`, attaching and
  detaching if the current thread was unattached. Never call JNI from the
  runtime's attach thread or hook threads; use `with_attached` on your own
  threads only.
- `cstr(s)` — null-terminated string for JNI `GetStaticMethodID` etc.

## method / static_method

```rust
classes::static_method(env, cls.as_jclass(), "hello", "(Ljava/lang/String;)V")?;
```

Name-based method lookup (no heavy initialization needed); a future SDK
version caches jmethodIDs per class.
