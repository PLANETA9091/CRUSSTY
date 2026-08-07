package ca.spottedleaf.moonrise.common.misc;

/**
 * Compile-time stubs so SingleUserAreaMapOps.java can be compiled standalone
 * (--release 8) against the real class shapes. NOT shipped: the real
 * classes come from the kernel jar / the crussty bridge injection.
 * Package-private on purpose (javac: one public top-level class per file).
 */
abstract class SingleUserAreaMap<T> {
    protected abstract void addCallback(T param, int chunkX, int chunkZ);
    protected abstract void removeCallback(T param, int chunkX, int chunkZ);
}

final class PaperNativeAreaMap {
    private PaperNativeAreaMap() {}
    static native int nativeUpdateOpsBatch(
        int fromX, int fromZ, int oldD, int toX, int toZ, int newD,
        byte[] ops, long[] keys
    );
}
