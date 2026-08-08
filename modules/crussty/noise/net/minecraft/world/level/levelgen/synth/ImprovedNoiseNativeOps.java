package net.minecraft.world.level.levelgen.synth;

/**
 * Native-noise bridge for the Crussty CE kernel patch.
 *
 * Replaces the body of {@code ImprovedNoise.noise(DDDDD)D} with a call to
 * {@link #noise} (invokestatic emitted by the byte hook, see
 * modules/crussty/src/improved_noise.rs). The rewritten body already
 * materialized the kernel fields it may touch legally — `p`/`xo`/`yo`/`zo`
 * are read inside ImprovedNoise's own method, so no access-flag changes are
 * needed on the class file.
 *
 * The bridge builds a native handle once per {@link ImprovedNoise} instance
 * (via {@link PaperNativeImprovedNoise#nativeBuildHandle}) and caches it in a
 * per-instance {@link Handle}; every sample then goes through
 * {@link PaperNativeImprovedNoise#nativeNoise}.
 *
 * Compile with `--release 8` (major 52) against the stub sources in
 * `stubs/` so the class loads via DefineClass into the kernel's class loader.
 * Generated artifacts live in `build/` (rebuild with scripts/build-helpers.sh,
 * keep in sync).
 */
public final class ImprovedNoiseNativeOps {
    private ImprovedNoiseNativeOps() {}

    /** Weak per-instance handle cache: no strong refs to kernel instances. */
    private static final java.util.Map<ImprovedNoise, Handle> HANDLES =
        new java.util.WeakHashMap<>();

    public static final class Handle {
        long nativeHandle;
    }

    /**
     * Bridge called from the patched {@code ImprovedNoise.noise(DDDDD)D}.
     *
     * @param self  the ImprovedNoise being sampled (handle cache key)
     * @param p     the kernel's permutation table (private field, already read)
     * @param xo    the kernel's octave offset X (private field, already read)
     * @param yo    the kernel's octave offset Y (private field, already read)
     * @param zo    the kernel's octave offset Z (private field, already read)
     * @param x     raw sample coordinate X
     * @param y     raw sample coordinate Y
     * @param z     raw sample coordinate Z
     * @param yScale per-octave y scaling
     * @param yMax  per-octave y bound
     */
    static double noise(
        ImprovedNoise self, byte[] p, double xo, double yo, double zo,
        double x, double y, double z, double yScale, double yMax
    ) {
        Handle h;
        synchronized (HANDLES) {
            h = HANDLES.get(self);
            if (h == null) {
                h = new Handle();
                HANDLES.put(self, h);
            }
        }
        if (h.nativeHandle == 0L) {
            h.nativeHandle = PaperNativeImprovedNoise.nativeBuildHandle(p, xo, yo, zo);
        }
        return PaperNativeImprovedNoise.nativeNoise(h.nativeHandle, x, y, z, yScale, yMax);
    }
}
