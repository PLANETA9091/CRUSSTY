package dev.dist;

/**
 * Eye bridge for the offline ImprovedNoise smoke: replaces the body of the
 * real kernel method ImprovedNoise.noise(DDDDD)D. Returns a constant — this
 * only proves the rewrite applies + the rewritten class loads + the method
 * runs; the production bridge adds the native handle dance via JNI.
 */
public final class NoiseBridge {
    public static double noise(double x, double y, double z, double yScale, double yMax) {
        return 0.123456789;
    }
}