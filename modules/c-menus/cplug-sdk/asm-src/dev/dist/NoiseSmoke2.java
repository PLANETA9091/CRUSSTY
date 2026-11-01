package dev.dist;

import java.io.ByteArrayOutputStream;
import java.io.DataOutputStream;
import java.lang.reflect.Field;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

/**
 * op2 (MAKE_FIELDS_PUBLIC) against the REAL kernel class: widens
 * ImprovedNoise.p from `private final byte[]` to `public final byte[]` so a
 * bridge class in a different class can read the permutation and build a
 * native handle. Loads the rewritten class in isolation and reflects on the
 * field to prove the access flag actually flipped.
 */
public final class NoiseSmoke2 {

    private static byte[] opPublicSpec() throws Exception {
        ByteArrayOutputStream bos = new ByteArrayOutputStream();
        DataOutputStream d = new DataOutputStream(bos);
        d.writeByte(1);   // version
        d.writeByte(2);   // op MAKE_FIELDS_PUBLIC
        d.writeByte(1);   // one field
        d.writeUTF("p");
        d.writeUTF("[B");
        d.flush();
        return bos.toByteArray();
    }

    public static void main(String[] args) throws Exception {
        byte[] cls = Files.readAllBytes(Paths.get("asm-src", "fixtures", "ImprovedNoise.class"));
        int major = ((cls[6] & 0xff) << 8) | (cls[7] & 0xff);
        System.out.println("fixture major=" + major + " (kernel)");

        byte[] pub = SdkAsmHelper.rewrite(cls, opPublicSpec());
        if (pub == null) {
            throw new IllegalStateException("op2 failed: " + SdkAsmHelper.lastError());
        }
        System.out.println("op2 MAKE_FIELDS_PUBLIC: " + cls.length + " -> " + pub.length + " bytes");

        Path outDir = Paths.get("asm-build", "rewritten-noise2");
        Path target = outDir.resolve("net/minecraft/world/level/levelgen/synth/ImprovedNoise.class");
        Files.createDirectories(target.getParent());
        Files.write(target, pub);
        ClassLoader loader = new java.net.URLClassLoader(
                new java.net.URL[] { outDir.toUri().toURL() },
                ClassLoader.getSystemClassLoader());
        Class<?> c = Class.forName("net.minecraft.world.level.levelgen.synth.ImprovedNoise",
                true, loader);
        Field p = c.getDeclaredField("p");
        int mods = p.getModifiers();
        boolean priv = (mods & 0x0002) != 0;
        boolean isPub = (mods & 0x0001) != 0;
        if (priv || !isPub) {
            throw new IllegalStateException("p modifiers = " + mods + " (expected public)");
        }
        Object inst = unsafe().allocateInstance(c);
        Object raw = p.get(inst);
        System.out.println("OK: p is public final, readable off-instance = " + raw);
    }

    private static sun.misc.Unsafe unsafe() throws Exception {
        java.lang.reflect.Field f = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
        f.setAccessible(true);
        return (sun.misc.Unsafe) f.get(null);
    }
}