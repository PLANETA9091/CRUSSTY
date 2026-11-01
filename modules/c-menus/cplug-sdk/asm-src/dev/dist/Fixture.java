package dev.dist;

/** Fixture: methods whose bodies we replace via SdkAsmHelper.rewrite. */
public final class Fixture {
    /** Original body (must be swapped out): x*100+7. */
    public static int id(int x) {
        return x * 100 + 7;
    }

    /** Bridge for id: returns 42 for any input. */
    public static int idBridge(int x) {
        return 42;
    }

    /** Two-slot args + long return. */
    public static long wide(long a, long b) {
        return a + b;
    }

    public static long wideBridge(long a, long b) {
        return -1L;
    }
}