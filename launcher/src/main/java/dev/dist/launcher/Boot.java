package dev.dist.launcher;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Enumeration;
import java.util.List;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;
import java.util.jar.Manifest;

/**
 * Single-jar entry point ("just drop it in place of server.jar").
 *
 * Bootstraps the native runtime from inside this jar, then hands control to
 * the kernel's main class. No -agentpath, no extra files to deploy:
 *
 *   java -Xmx2G -jar crussty-server.jar --nogui
 *
 * Sequence:
 *   1. extract the crussty runtime (libcrussty_runtime.so / .dylib /
 *      crussty_runtime.dll) and every embedded module into ./crussty/;
 *   2. write crussty/options.txt (modules=...) for the runtime;
 *   3. System.load the runtime -> JNI_OnLoad brings up the JVMTI pipeline
 *      before any kernel class loads;
 *   4. invoke the kernel's main class reflectively.
 *
 * Works with ANY bootable server jar, not just paperclip kernels: the
 * builder (`crussty build`) stores the kernel's original Main-Class in the
 * merged manifest under "Crussty-Kernel-Main" and Boot invokes that. When
 * the attribute is missing (jars assembled by hand or by the legacy
 * build-single-jar.sh), Boot falls back to io.papermc.paperclip.Main —
 * Paper/Purpur kernels — exactly like the previous hard-wired behaviour.
 *
 * This class is compiled with `javac --release 8` (see
 * scripts/build-helpers.sh) and the .class artifact is COMMITTED: the CLI
 * embeds it byte-for-byte via include_bytes!, so `crussty build` works on
 * machines without any Java toolchain. Keep the source Java-8 compatible
 * (no transferTo/Files.writeString/List.of/…).
 */
public final class Boot {
    private static final String STAGE = "crussty";

    /** Manifest attribute the builder writes: the kernel's original Main-Class. */
    static final String KERNEL_MAIN_ATTR = "Crussty-Kernel-Main";

    /** Fallback when the attribute is absent (legacy paperclip-only jars). */
    static final String PAPERCLIP_MAIN = "io.papermc.paperclip.Main";

    /** Runtime library names Boot recognizes at the jar root, per OS. */
    private static final String[] RUNTIME_NAMES = {
        "libcrussty_runtime.so", "libcrussty_runtime.dylib", "crussty_runtime.dll",
    };

    private Boot() {}

    /** Native runtime library name for the current OS. */
    private static String runtimeLibName() {
        String os = System.getProperty("os.name", "").toLowerCase();
        if (os.contains("win")) {
            return "crussty_runtime.dll";
        }
        if (os.contains("mac")) {
            return "libcrussty_runtime.dylib";
        }
        return "libcrussty_runtime.so";
    }

    private static boolean isRuntimeName(String name) {
        for (String n : RUNTIME_NAMES) {
            if (n.equals(name)) {
                return true;
            }
        }
        return false;
    }

    public static void main(String[] args) throws Exception {
        Path stage = Paths.get(STAGE);
        Files.createDirectories(stage);

        List<String> modules = new ArrayList<>();
        Path runtime = extractResources(stage, modules);
        if (runtime == null) {
            System.err.println("[crussty] no crussty runtime embedded in this jar —"
                    + " rebuild it with `crussty build`");
            System.exit(2);
            return;
        }
        writeOptions(stage, modules);
        System.load(runtime.toAbsolutePath().toString());
        System.out.println("[crussty] runtime loaded from " + runtime);

        // The kernel classes live in this same jar (merged by the builder);
        // paperclip will download the vanilla jar and unpack libraries on
        // first run, exactly like a stock kernel would.
        invokeKernelMain(args);
    }

    /** Reflectively call the kernel's main: manifest Crussty-Kernel-Main, else paperclip. */
    private static void invokeKernelMain(String[] args) throws Exception {
        String main = kernelMainClass();
        System.out.println("[crussty] kernel main: " + main);
        Class<?> clazz = Class.forName(main);
        Method mainMethod = clazz.getMethod("main", String[].class);
        mainMethod.invoke(null, new Object[] { args });
    }

    /** Read Crussty-Kernel-Main from this jar's manifest (paperclip fallback). */
    private static String kernelMainClass() {
        String jarPath = jarPath();
        if (jarPath != null) {
            try (JarFile jar = new JarFile(jarPath)) {
                Manifest mf = jar.getManifest();
                if (mf != null) {
                    String v = mf.getMainAttributes().getValue(KERNEL_MAIN_ATTR);
                    if (v != null && !v.trim().isEmpty()) {
                        return v.trim();
                    }
                }
            } catch (IOException ignored) {
                // fall through to the paperclip default
            }
        }
        return PAPERCLIP_MAIN;
    }

    /** Location of this jar (Boot.class code source, %20 unescaped). */
    private static String jarPath() {
        try {
            return Boot.class.getProtectionDomain()
                    .getCodeSource().getLocation().getPath().replace("%20", " ");
        } catch (Exception e) {
            return null;
        }
    }

    /** Extract the runtime library and every module file embedded in the jar.
     *  Returns the extracted runtime library path, or null when none embedded. */
    private static Path extractResources(Path stage, List<String> modules) throws IOException {
        Path runtime = null;
        String preferred = runtimeLibName();
        String jarPath = jarPath();
        if (jarPath == null) {
            return null;
        }
        try (JarFile jar = new JarFile(jarPath)) {
            Enumeration<JarEntry> entries = jar.entries();
            while (entries.hasMoreElements()) {
                JarEntry e = entries.nextElement();
                String name = e.getName();
                if (e.isDirectory()) {
                    continue;
                }
                if (isRuntimeName(name)) {
                    Path out = stage.resolve(name);
                    extract(jar, e, out);
                    if (runtime == null || name.equals(preferred)) {
                        runtime = out;
                    }
                } else if (name.startsWith("modules/")) {
                    Path out = stage.resolve(name);
                    Files.createDirectories(out.getParent());
                    extract(jar, e, out);
                    String dir = out.getParent().getFileName().toString();
                    if (!modules.contains(dir)) {
                        modules.add(dir);
                    }
                }
            }
        }
        return runtime;
    }

    private static void extract(JarFile jar, JarEntry e, Path out) throws IOException {
        byte[] buf = new byte[8192];
        try (InputStream in = jar.getInputStream(e);
             OutputStream os = Files.newOutputStream(out)) {
            int n;
            while ((n = in.read(buf)) != -1) {
                os.write(buf, 0, n);
            }
        }
    }

    /** options format understood by the runtime: "modules=<dir>;..." */
    private static void writeOptions(Path stage, List<String> modules) throws IOException {
        String modulesDir = stage.resolve("modules").toAbsolutePath().toString();
        Files.write(stage.resolve("options.txt"),
                ("modules=" + modulesDir + "\n").getBytes("UTF-8"));
        if (modules.isEmpty()) {
            System.out.println("[crussty] no modules embedded, nothing to inject");
        } else {
            System.out.println("[crussty] embedded modules: " + String.join(", ", modules));
        }
    }
}
