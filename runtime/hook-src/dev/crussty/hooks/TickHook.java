package dev.crussty.hooks;

/**
 * Generic ()V entry hook used by the transform engine's default test hook.
 * The runtime registers onEntry against a no-op so classes patched by
 * engine-level tests or lightweight rules resolve without linkage errors.
 */
public final class TickHook {
    private TickHook() {}

    public static native void onEntry();
}