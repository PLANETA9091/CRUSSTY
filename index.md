---
layout: home
title: Crussty Platform
nav_order: 1
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/datapacks.svg" alt=""> Crussty Platform


Crussty is a **native injection platform** for Paper-family Minecraft kernels
(Purpur, Paper, etc.). It attaches a JVMTI runtime to the kernel JVM and uses
it to load **Rust modules** — shared libraries written entirely in Rust —
directly into the server process, where they can intercept class loading,
patch kernel bytecode, and replace hot paths with native code.

Call it what you like: an injector, a hot-patch engine, a native module
platform. The distinguishing property is that Crussty does not fork the
kernel and does not rebuild it — it injects itself into a stock jar and
changes behavior at class-load time.


## What it does 

A JVMTI runtime (`libcrussty_runtime.so`) is attached to the kernel JVM with
`-agentpath` (or loaded via `JNI_OnLoad` in the single-jar distribution). It:

- scans a `modules/` directory for modules,
- loads each module (`dlopen`, `RTLD_LOCAL`) in dependency order,
- forwards every class load through the module hook pipeline
  (`CLASS_FILE_LOAD_HOOK`), so modules can patch kernel bytecode on the fly
  (hot-patching),
- lets modules retransform already-loaded classes.

Modules get a `JavaVM*` and can do anything JNI/JVMTI allows: resolve classes,
run code on the server main thread, call Bukkit APIs, rewrite bytecode with
ASM.

## Why native

Rust modules run without the JVM in the hot path: no garbage collection, no
interpreter, no classloading — just compiled code. This is what makes
hot-path replacements (worldgen noise, chunk encoding, block collisions)
thousands of times faster than Java equivalents.

## Build your own module

Want to write a module? Follow the step-by-step
[Building an example module](modules/creating/rust/example.html) guide — it takes a
module from zero to a running server in seven steps. Or grab a ready-made
module from the [example repos](modules/examples.html).
