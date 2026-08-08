package dev.dist.launcher;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;

/**
 * Single-jar entry point ("just drop it in place of server.jar").
 *
 * Bootstraps the native runtime from inside this jar, then hands control to
 * the kernel's paperclip main. No -agentpath, no extra files to deploy:
 *
 *   java -Xmx2G -jar server.jar --nogui
 *
 * Sequence:
 *   1. extract libcrussty_runtime.so and modules/* into ./crussty/;
 *   2. write crussty/options.txt (modules=...) for the runtime;
 *   3. System.load the runtime -> JNI_OnLoad brings up the JVMTI pipeline
 *      before any kernel class loads;
 *   4. run io.papermc.paperclip.Main (the kernel is merged into this jar).
 */
public final class Boot {
    private static final String STAGE = "crussty";

    private Boot() {}

    /** Native runtime library name for the current OS (crussty_runtime.dll on Windows). */
    private static String runtimeLibName() {
        return System.getProperty("os.name", "").toLowerCase().contains("win")
                ? "crussty_runtime.dll" : "libcrussty_runtime.so";
    }

    public static void main(String[] args) throws Exception {
        Path stage = Paths.get(STAGE);
        Files.createDirectories(stage);

        List<String> modules = extractResources(stage);
        writeOptions(stage, modules);

        Path runtime = stage.resolve(runtimeLibName());
        System.load(runtime.toAbsolutePath().toString());
        System.out.println("[crussty] runtime loaded from " + runtime);

        // The kernel classes (io.papermc.paperclip.*) live in this same jar;
        // paperclip will download the vanilla jar and unpack libraries on
        // first run, exactly like a stock kernel would.
        io.papermc.paperclip.Main.main(args);
    }

    /** Extract the runtime library and every module .so/.dll embedded in the jar. */
    private static List<String> extractResources(Path stage) throws IOException {
        List<String> modules = new ArrayList<>();
        String jarPath = Boot.class.getProtectionDomain()
                .getCodeSource().getLocation().getPath().replace("%20", " ");
        try (JarFile jar = new JarFile(jarPath)) {
            java.util.Enumeration<JarEntry> entries = jar.entries();
            while (entries.hasMoreElements()) {
                JarEntry e = entries.nextElement();
                String name = e.getName();
                if (name.equals(runtimeLibName())) {
                    extract(jar, e, stage.resolve(runtimeLibName()));
                } else if (name.startsWith("modules/") && !name.endsWith("/")) {
                    Path out = stage.resolve(name);
                    Files.createDirectories(out.getParent());
                    extract(jar, e, out);
                    modules.add(out.getParent().getFileName().toString());
                }
            }
        }
        return modules;
    }

    private static void extract(JarFile jar, JarEntry e, Path out) throws IOException {
        try (InputStream in = jar.getInputStream(e);
             OutputStream os = Files.newOutputStream(out)) {
            in.transferTo(os);
        }
    }

    /** options format understood by the runtime: "modules=<dir>;..." */
    private static void writeOptions(Path stage, List<String> modules) throws IOException {
        String modulesDir = stage.resolve("modules").toAbsolutePath().toString();
        Files.writeString(stage.resolve("options.txt"),
                "modules=" + modulesDir + "\n");
        if (modules.isEmpty()) {
            System.out.println("[crussty] no modules embedded, nothing to inject");
        } else {
            System.out.println("[crussty] embedded modules: " + String.join(", ", modules));
        }
    }
}
