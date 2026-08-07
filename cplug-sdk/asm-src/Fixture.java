package dev.dist;

/** Fixture: a method whose body we replace via SdkAsmHelper.rewrite. */
public final class Fixture {
    public static int id(int x) {
        return x * 100 + 7; // original body, must be swapped out
    }
    public static long wide(long a, long b) {
        return a + b; // two-slot args, long return
    }
    public static void main(String[] args) {
        System.out.println("original id(3)=" + id(3) + " wide(2,3)=" + wide(2L, 3L));
    }
}