package dev.crussty.hooks;

/**
 * Native bridge required by the network-interception transform rules
 * (runtime/src/platform/network.rs). The rule-injected ()V probes cannot
 * carry connection ids or packets; the runtime registers these natives
 * against no-ops until module tooling wires the argument-carrying surface.
 */
public final class NetHooks {
    private NetHooks() {}

    public static native void onDecode();

    public static native void onEncode();

    public static native void onIntention();

    public static native void onProtocolSwap();

    public static native void onChannelInactive();
}