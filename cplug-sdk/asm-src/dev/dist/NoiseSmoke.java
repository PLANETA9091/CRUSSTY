package dev.dist;

import java.io.ByteArrayOutputStream;
import java.io.DataOutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.lang.reflect.Method;

/**
 * Offline smoke against the REAL kernel class: rewrites
 * ImprovedNoise.noise(DDDDD)D to call dev.dist.NoiseBridge.noise(DDDDD)D,
 * then loads the rewritten bytes with a dedicated ClassLoader (no access to
 * the original) and invokes the patched method. This exercises the whole M3
 * pipeline — ASM rewrite, frame recomputation, defineClass, the JVM verifier —
 * against actual Purpur 1.21.10 bytecode (major 65) without a server.
 *
 * The patched method is an INSTANCE method; we build an instance without
 * running the kernel constructor via Unsafe.allocateInstance (constructor
 * needs a RandomSource we don't have offline).
 */
public final class NoiseSmoke {

    private static byte[] spec() throws Exception {
        ByteArrayOutputStream bos = new ByteArrayOutputStream();
        DataOutputStream d = new DataOutputStream(bos);
        d.writeByte(1);              // spec version
        d.writeByte(1);              // op: REPLACE_BODY
        d.writeUTF("noise");         // methodName
        d.writeUTF("(DDDDD)D");      // methodDesc
        d.writeUTF("dev/dist/NoiseBridge"); // bridgeOwner
        d.writeUTF("noise");         // bridgeName
        d.writeUTF("(DDDDD)D");      // bridgeDesc
        d.writeByte(5);              // argCount: the five doubles
        for (int slot = 1; slot <= 9; slot += 2) {
            d.writeShort(slot);      // DLOAD slot
            d.writeByte('D');
        }
        d.flush();
        return bos.toByteArray();
    }

    public static void main(String[] args) throws Exception {
        byte[] cls = Files.readAllBytes(Paths.get("asm-src", "fixtures", "ImprovedNoise.class"));
        int magic = ((cls[0] & 0xff) << 24) | ((cls[1] & 0xff) << 16) | ((cls[2] & 0xff) << 8) | (cls[3] & 0xff);
        if (magic != 0xCAFEBABE) {
            throw new IllegalStateException("fixture not a classfile");
        }
        int major = ((cls[6] & 0xff) << 8) | (cls[7] & 0xff);
        System.out.println("fixture major=" + major);

        byte[] out = SdkAsmHelper.rewrite(cls, spec());
        if (out == null) {
            throw new IllegalStateException("rewrite failed: " + SdkAsmHelper.lastError());
        }
        System.out.println("rewritten " + cls.length + " -> " + out.length + " bytes");

        Path outDir = Paths.get("asm-build", "rewritten-noise");
        Path target = outDir.resolve("net/minecraft/world/level/levelgen/synth/ImprovedNoise.class");
        Files.createDirectories(target.getParent());
        Files.write(target, out);

        // Load the REWRITTEN class with a loader that does NOT see the
        // original (parent = platform classloader). The bridge stays on the
        // application classpath so the invokestatic resolves.
        ClassLoader loader = new java.net.URLClassLoader(
                new java.net.URL[] { outDir.toUri().toURL() },
                ClassLoader.getSystemClassLoader());
        Class<?> c = Class.forName("net.minecraft.world.level.levelgen.synth.ImprovedNoise",
                true, loader);
        System.out.println("loaded " + c.getName() + " from " + c.getClassLoader());

        // instance without constructor
        Object inst = unsafe().allocateInstance(c);

        Method noise5 = c.getMethod("noise",
                double.class, double.class, double.class, double.class, double.class);
        double r = (Double) noise5.invoke(inst, 1.0, 2.0, 3.0, 0.5, 1.0);
        if (Math.abs(r - 0.123456789) > 1e-12) {
            throw new IllegalStateException("patched noise returned " + r);
        }
        System.out.println("OK: patched noise(1,2,3,0.5,1)=" + r + " (bridge body, verifier passed)");
    }

    private static sun.misc.Unsafe unsafe() throws Exception {
        java.lang.reflect.Field f = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
        f.setAccessible(true);
        return (sun.misc.Unsafe) f.get(null);
    }
}