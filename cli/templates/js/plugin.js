// __NAME__ — module body written in JavaScript (loaded by the QuickJS shim).
//
// QuickJS embedded bare has no `console` (that lives in quickjs-libc, which
// this shim does not link); the shim exposes `logNative(msg)` instead.

function make_hook() {
    // Called by the C shim; returns the per-class-load callback.
    return function on_class_load(name) {
        logNative("[__NAME__] hook fired for " + name);
        return null; // keep original bytes
    };
}