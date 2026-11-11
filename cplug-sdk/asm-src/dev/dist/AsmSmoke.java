package dev.dist;

import java.io.ByteArrayOutputStream;
import java.io.DataOutputStream;
import java.nio.file.Files;
import java.nio.file.Paths;

/**
 * Offline smoke test for SdkAsmHelper (no JVM agent needed):
 * rewrites Fixture.id(I)I's body to a call of Fixture.idBridge(I)I and
 * Fixture.wide(JJ)J to wideBridge(JJ)J, reloads the rewritten classes and
 * verifies the bridge bodies actually ran (verifier must accept the
 * rewritten bytecode — COMPUTE_FRAMES output).
 */
public final class AsmSmoke {
    public static void main(String[] args) throws Exception {
        byte[] cls = Files.readAllBytes(Paths.get("asm-build", "dev", "dist", "Fixture.class"));

        byte[] specId = spec("id", "(I)I", "dev/dist/Fixture", "idBridge", "(I)I",
                new int[][] { { 0, 'I' } });
        byte[] rwId = SdkAsmHelper.rewrite(cls, specId);
        if (rwId == null) {
            System.err.println("FAIL id: " + SdkAsmHelper.lastError());
            System.exit(1);
        }
        Files.createDirectories(Paths.get("asm-build", "rewritten", "dev", "dist"));
        Files.write(Paths.get("asm-build", "rewritten", "dev", "dist", "Fixture.class"), rwId);

        byte[] specWide = spec("wide", "(JJ)J", "dev/dist/Fixture", "wideBridge", "(JJ)J",
                new int[][] { { 0, 'J' }, { 2, 'J' } });
        byte[] rwWide = SdkAsmHelper.rewrite(cls, specWide);
        if (rwWide == null) {
            System.err.println("FAIL wide: " + SdkAsmHelper.lastError());
            System.exit(1);
        }
        Files.createDirectories(Paths.get("asm-build", "rewritten-w", "dev", "dist"));
        Files.write(Paths.get("asm-build", "rewritten-w", "dev", "dist", "Fixture.class"), rwWide);

        // The rewritten class keeps its declared name "dev/dist/Fixture"; load each
        // rewritten copy from a fresh URLClassLoader whose parent CANNOT see
        // the original (else the parent-first delegation picks the untouched
        // bytecode). The rewritten class is self-contained (its bridge is a
        // sibling method in the same class), so the child loader suffices.
        java.net.URL rwIdUrl = new java.io.File("asm-build", "rewritten").toURI().toURL();
        Class<?> idC = new java.net.URLClassLoader(
                new java.net.URL[] { rwIdUrl }, null)
                .loadClass("dev.dist.Fixture");
        Object idV = idC.getMethod("id", int.class).invoke(null, 3);
        if (!idV.equals(42)) {
            System.err.println("FAIL: Fixture.id(3)=" + idV + " (expect 42: bridge body)");
            System.exit(1);
        }
        java.net.URL rwWideUrl = new java.io.File("asm-build", "rewritten-w").toURI().toURL();
        Class<?> wideC = new java.net.URLClassLoader(
                new java.net.URL[] { rwWideUrl }, null)
                .loadClass("dev.dist.Fixture");
        Object wideV = wideC.getMethod("wide", long.class, long.class).invoke(null, 2L, 3L);
        if (!wideV.equals(-1L)) {
            System.err.println("FAIL: Fixture.wide(2,3)=" + wideV + " (expect -1: bridge body)");
            System.exit(1);
        }
        System.out.println("OK: id(3)=42, wide(2,3)=-1 — bridge bodies replaced, verifier passed");
    }

    static byte[] spec(String methodName, String methodDesc, String owner, String bridgeName,
            String bridgeDesc, int[][] slots) throws Exception {
        ByteArrayOutputStream b = new ByteArrayOutputStream();
        DataOutputStream d = new DataOutputStream(b);
        d.writeByte(1); // version
        d.writeByte(1); // op REPLACE_BODY
        d.writeUTF(methodName);
        d.writeUTF(methodDesc);
        d.writeUTF(owner);
        d.writeUTF(bridgeName);
        d.writeUTF(bridgeDesc);
        d.writeByte(slots.length);
        for (int[] s : slots) {
            d.writeShort(s[0]);
            d.writeByte(s[1]);
        }
        return b.toByteArray();
    }
}