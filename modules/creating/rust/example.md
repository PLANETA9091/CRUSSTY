---
title: Building an example module
parent: Rust
nav_order: 1
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/rust.svg" alt="Rust"> Building an example module

This guide builds a working module — the `hello` proof module — from scratch,
step by step. You will have it running on a server in about five minutes.

<div class="ws-try">
  <a class="ws-btn" href="https://github.com/PLANETA9091/c-hello/releases">Try it yourself — download the finished module</a>
  <a class="ws-btn-ghost" href="https://github.com/PLANETA9091/c-hello">Browse the source</a>
  <div class="ws-try-note">Already running a platform? Skip straight to
  <a href="#step-6-install-the-module">Step 6</a> — grab a <code>.zip</code> from the
  <a href="https://github.com/PLANETA9091/c-hello/releases">c-hello release</a> and drop it into <code>modules/</code>.</div>
</div>

## What a module is

A module is a shared library that exports **one** C-ABI function:

```c
int32_t cplugin_init(const CPluginApi* api, JavaVM* vm, const char* options);
```

The runtime scans `modules/`, dlopens each `<id>/lib<id>.so`, calls
`cplugin_init`, and from then on your module receives class-load hook calls,
main-thread callbacks and JNI access. That's the whole contract — anything
that can export this function (see [JavaScript](../javascript.html), [Python](../python.html), [C & C++](../c.html))
can be a module.

---

<div class="ws-step">
<h2><span class="step-no">1.</span> Create the folder</h2>

Modules live inside the platform tree, next to `cplug-abi` and `cplug-sdk`:

```bash
cd <crussty>
mkdir -p modules/hello/src
```

```
crussty/
├── cplug-abi/        # C ABI (path dependency)
├── cplug-sdk/        # Rust SDK (path dependency)
├── runtime/          # the platform itself
└── modules/
    └── hello/        # ← your module
        ├── cplugin.json
        ├── Cargo.toml
        └── src/
            └── lib.rs
```
</div>

<div class="ws-step">
<h2><span class="step-no">2.</span> Write the manifest</h2>

`cplugin.json` tells the runtime your module exists and what its entry library
is called:

```json
{"id": "hello", "version": "0.1.0"}
```

- `id` — unique module id; the runtime looks for `lib<id>.so` (here `libhello.so`)
- `version` — optional metadata; the loader ignores it (only `id`/`main`/`dependencies` are read)

> **Rename rule:** if your library is *not* named `lib<id>.so`, set the
> explicit entry name: `{"id": "hello", "main": "libmycustom.so"}`.
</div>

<div class="ws-step">
<h2><span class="step-no">3.</span> Configure Cargo</h2>

The module is a `cdylib` (shared library) with two path dependencies:

```toml
[package]
name = "hello"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
cplug-abi = { path = "../../cplug-abi" }
cplug-sdk = { path = "../../cplug-sdk" }
```

> The paths are relative to `modules/hello` inside the platform tree.
</div>

<div class="ws-step">
<h2><span class="step-no">4.</span> Write the module code</h2>

Three files make up the module body. Click the tabs:

<div class="ws-tabs">
  <div class="ws-tabbar">
    <button class="active" data-tab="tab-manifest">cplugin.json</button>
    <button data-tab="tab-cargo">Cargo.toml</button>
    <button data-tab="tab-lib">src/lib.rs</button>
  </div>
  <div class="ws-tab active" id="tab-manifest">
<pre><code>{"id": "hello", "version": "0.1.0"}</code></pre>
  </div>
  <div class="ws-tab" id="tab-cargo">
<pre><code>[package]
name = "hello"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
cplug-abi = { path = "../../cplug-abi" }
cplug-sdk = { path = "../../cplug-sdk" }</code></pre>
  </div>
  <div class="ws-tab" id="tab-lib">
<pre><code>use cplug_abi::{CPluginApi, JavaVmPtr};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cplugin_init(
    api: *const CPluginApi,
    vm: JavaVmPtr,
    _options: *const std::ffi::c_char,
) -> i32 {
    // First log line — the runtime calls this once at scan time, before
    // the kernel boots.
    eprintln!("[hello-plugin] cplugin_init (native, before kernel boot)");

    cplug_sdk::init(api, vm);

    // Fire when org/bukkit/Bukkit is loaded — proves the hook chain.
    cplug_sdk::hooks::register("org/bukkit/Bukkit", |_name| {
        eprintln!("[hello-plugin] Bukkit class load observed (hook chain ok)");
    });

    // Once the kernel is up, log through Bukkit.getLogger() from the
    // main thread (the queue flushes only after the server object exists).
    cplug_sdk::on_kernel_ready("org.bukkit.Bukkit", || {
        let found = cplug_sdk::classes::find_class("org/bukkit/Bukkit").is_ok();
        eprintln!("[hello-plugin] GetLoadedClasses resolved Bukkit: {}", found);
        cplug_sdk::run_on_main_thread(|_env| {
            cplug_sdk::log::info("hello from native c-plugin (v2 pipeline alive)");
        });
    });
    0
}</code></pre>
  </div>
</div>

What each part does:

- `cplugin_init` — the required entry point; the runtime calls it once at scan time
- `cplug_sdk::init(api, vm)` — hands the SDK the platform pointers
- `hooks::register` — subscribes to the class-file hook pipeline; your
  callback runs on the class-load thread whenever that class is defined
- `on_kernel_ready` + `run_on_main_thread` — queues a JNI call on the kernel's
  main thread, once the server is actually up
- `log::info` — `Bukkit.getLogger().info(...)` through JNI
</div>

<div class="ws-step">
<h2><span class="step-no">5.</span> Build</h2>

```bash
cd modules/hello
cargo build
cp target/debug/libhello.so libhello.so
```

You should now have three files in `modules/hello/`:

```text
modules/hello/
├── cplugin.json
├── libhello.so
└── src/lib.rs
```

To hand the module to someone who doesn't build, pack it:

```bash
zip -r hello.zip cplugin.json libhello.so
```
</div>

<div class="ws-step">
<h2><span class="step-no">6.</span> Install the module</h2>

Either unpack a module zip:

```bash
cd <crussty>
unzip hello.zip -d modules/hello
```

or place the files directly:

```bash
mkdir -p modules/hello
cp libhello.so cplugin.json modules/hello/
```

That's it — no config, no registration. The runtime rescans `modules/`
recursively on every boot and picks up anything new.
</div>

<div class="ws-step">
<h2><span class="step-no">7.</span> Run and verify</h2>

Boot the platform and watch the log:

```text
[crussty-runtime] scanning modules/ ...
[crussty-runtime] module hello -> manifest ok, entry libhello.so
[hello-plugin] cplugin_init (native, before kernel boot)
[hello-plugin] Bukkit class load observed (hook chain ok)
[hello-plugin] GetLoadedClasses resolved Bukkit: true
[13:36:26 INFO]: hello from native c-plugin (v2 pipeline alive)
```

The last line — `hello from native c-plugin` — means the full pipeline works:
scan → load → hook → main-thread bridge → JNI.

<div class="ws-try">
  <a class="ws-btn" href="https://github.com/PLANETA9091/c-hello/releases">Try it yourself</a>
  <a class="ws-btn-ghost" href="https://github.com/PLANETA9091/c-hello">Clone the repo</a>
  <div class="ws-try-note">Stuck? See
  <a href="../../../troubleshooting.html">Troubleshooting</a>, or start from the ready-made
  <a href="../../../modules/examples.html">example modules</a>.</div>
</div>
</div>

## Next steps

- [Modules](../../../modules.html) — the full module contract
- [Module SDK](../../../sdk.html) — hooks, classes, main thread, ASM
- [JavaScript](../javascript.html), [Python](../python.html), [C & C++](../c.html) — the same module in C, C++, Python, JS (Go caveat)
- [Troubleshooting](../../../troubleshooting.html) — common issues

