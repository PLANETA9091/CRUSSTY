package dev.crussty.hooks;

/**
 * Native bridge required by the network-interception transform rules
 * (runtime/src/platform/network.rs).
 *
 * The engine injects `aload <slot>...; invokestatic` at method entry, so the
 * frame's own values arrive here as arguments — no frame walk, no JVMTI
 * capability, a few nanoseconds per packet:
 *
 * <pre>
 *   PacketDecoder.decode(ChannelHandlerContext ctx, ByteBuf in, List out)
 *       -> onDecode(ctx, in)
 *   PacketEncoder.encode(ChannelHandlerContext ctx, Packet msg, ByteBuf out)
 *       -> onEncode(ctx, msg)
 *   ServerHandshakePacketListenerImpl.handleIntention(ClientIntentionPacket p)
 *       -> onIntention(p)
 *   Connection.setupInboundProtocol(ProtocolInfo protocol, PacketListener listener)
 *       -> onProtocolSwap(protocol, listener)
 *   Connection.channelInactive(ChannelHandlerContext ctx)
 *       -> onChannelInactive(ctx)
 * </pre>
 *
 * Parameters are declared as {@code Object} on purpose: the injected descriptor
 * is {@code (Ljava/lang/Object;...)V}, which stays valid whatever the kernel's
 * naming namespace or class hierarchy is.
 */
public final class NetHooks {
    private NetHooks() {}

    public static native void onDecode(Object ctx, Object buf);

    public static native void onEncode(Object ctx, Object msg);

    public static native void onIntention(Object packet);

    public static native void onProtocolSwap(Object protocol, Object listener);

    public static native void onChannelInactive(Object ctx);
}
