package net.minecraft.world.level.levelgen.synth;

/**
 * Compile-time stub for the native improved-noise bridge. NOT shipped: the
 * real class lives in the Paper family kernel (paper-native-jni) and is
 * resolved at runtime from the kernel's class loader.
 */
public final class PaperNativeImprovedNoise {
    private PaperNativeImprovedNoise() {}

    public static native long nativeBuildHandle(byte[] p, double xo, double yo, double zo);

    public static native double nativeNoise(
        long handle, double x, double y, double z, double yScale, double yMax
    );

    public static native void nativeFreeHandle(long handle);
}