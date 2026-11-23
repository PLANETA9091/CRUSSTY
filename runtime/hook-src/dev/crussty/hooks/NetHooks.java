package dev.crussty.hooks;

import java.lang.reflect.Method;

/**
 * Native bridge required by the network-interception transform rules
 * (runtime/src/platform/network.rs). The rule-injected ()V probes cannot
 * carry connection ids or packets; the runtime registers these natives
 * against no-ops until module tooling wires the argument-carrying surface.
 *
 * {@link #onEncodeExit(Object, Object)} is the outbound write-back probe
 * (slice 2): injected before every return of PacketEncoder.encode. It is
 * bootstrap-defined, so Netty types are reached only through reflection on
 * the buffer's own class loader; the native side sees plain byte arrays.
 */
public final class NetHooks {
    private NetHooks() {}

    public static native void onDecode();

    public static native void onEncode();

    public static native void onIntention();

    public static native void onProtocolSwap();

    public static native void onChannelInactive();

    /** True while at least one rewrite hook is registered (one atomic load). */
    private static native boolean rewriteLive();

    /** Rewrites one frame; returns null when no hook changed it. */
    private static native byte[] rewriteOutbound(byte[] payload);

    /*
     * Netty ByteBuf accessors, resolved once per buffer implementation and
     * published as one array: index 0 = implementation class, then the
     * interface methods in ACC_* order. No nested classes: the runtime embeds
     * only NetHooks.class (runtime/src/platform/hooks.rs).
     */
    private static final int ACC_IMPL = 0, ACC_READER_IDX = 1, ACC_READABLE = 2,
            ACC_GET_BYTES = 3, ACC_WRITER_IDX = 4, ACC_WRITE_BYTES = 5;

    private static volatile Object[] accessors;
    private static volatile boolean reported;

    /**
     * Exit probe for PacketEncoder.encode: {@code ctx} is the channel context,
     * {@code buf} the outbound frame buffer. Never throws into the encoder; on
     * any failure the buffer is left as the encoder wrote it.
     */
    public static void onEncodeExit(Object ctx, Object buf) {
        if (buf == null) {
            return;
        }
        try {
            if (!rewriteLive()) {
                return;
            }
            Object[] a = accessors(buf);
            int r = (Integer) ((Method) a[ACC_READER_IDX]).invoke(buf);
            int n = (Integer) ((Method) a[ACC_READABLE]).invoke(buf);
            byte[] payload = new byte[n];
            ((Method) a[ACC_GET_BYTES]).invoke(buf, Integer.valueOf(r), payload);
            byte[] out = rewriteOutbound(payload);
            if (out == null) {
                return;
            }
            // Whole-frame replacement: the encoder's output is the complete
            // unprefixed packet, so the new frame may differ in length.
            ((Method) a[ACC_WRITER_IDX]).invoke(buf, Integer.valueOf(r));
            ((Method) a[ACC_WRITE_BYTES]).invoke(buf, (Object) out);
        } catch (Throwable t) {
            if (!reported) {
                reported = true;
                System.err.println("[crussty-runtime] onEncodeExit rewrite skipped: " + t);
            }
        }
    }

    private static Object[] accessors(Object buf) throws Exception {
        Object[] a = accessors;
        if (a == null || a[ACC_IMPL] != buf.getClass()) {
            ClassLoader cl = buf.getClass().getClassLoader();
            Class<?> bb = Class.forName("io.netty.buffer.ByteBuf", false, cl);
            a = new Object[6];
            a[ACC_IMPL] = buf.getClass();
            a[ACC_READER_IDX] = bb.getMethod("readerIndex");
            a[ACC_READABLE] = bb.getMethod("readableBytes");
            a[ACC_GET_BYTES] = bb.getMethod("getBytes", int.class, byte[].class);
            a[ACC_WRITER_IDX] = bb.getMethod("writerIndex", int.class);
            a[ACC_WRITE_BYTES] = bb.getMethod("writeBytes", byte[].class);
            accessors = a;
        }
        return a;
    }
}
