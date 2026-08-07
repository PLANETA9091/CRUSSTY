package dev.dist.launcher;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.stream.Stream;

/**
 * v2 launcher (the artifact users download). Jobs:
 *  1. find the kernel jar in versions/ (any Paper-family jar);
 *  2. spawn a child JVM with our native agent (-agentpath:libdist_agent.so),
 *     so Rust modules load BEFORE the kernel boots (JVMTI ClassFileLoadHook
 *     window, full capabilities — OnLoad-only hooks are available);
 *  3. forward console stdin and tee stdout/stderr into logs/.
 *
 * We deliberately do NOT bootstrap the kernel in-process (Paperclip style):
 * child-JVM gives classpath isolation, crash isolation, cheap restarts and —
 * critically — an agent attached before any class loads (in-process
 * URLClassLoader can never obtain Instrumentation without a startup flag).
 */
public final class Main {

    public static void main(String[] args) throws Exception {
        Path root = Path.of(System.getProperty("dist.root", Path.of("").toAbsolutePath().toString()));
        Path versions = root.resolve("versions");
        Path modules = root.resolve("modules");
        Path logs = root.resolve("logs");

        Path kernel = findKernel(versions);
        Path agent = findAgent(root);

        Files.createDirectories(logs);
        String opts = "modules=" + modules.toAbsolutePath()
                + ";versions=" + versions.toAbsolutePath()
                + ";kernel=" + kernel.getFileName();

        List<String> cmd = new ArrayList<>();
        cmd.add(javaBin());
        cmd.add("-agentpath:" + agent.toAbsolutePath() + "=" + opts);
        cmd.addAll(javaOpts());
        // The child JVM must see the same dist root the launcher resolved
        // (else modules/kernel resolution differs when launched from elsewhere).
        cmd.add("-Ddist.root=" + root.toAbsolutePath());
        cmd.add("-jar");
        cmd.add(kernel.toAbsolutePath().toString());
        cmd.add("--nogui");
        cmd.addAll(List.of(args));

        System.out.println("[launcher] kernel:  " + kernel);
        System.out.println("[launcher] agent:   " + agent);
        System.out.println("[launcher] modules: " + modules + " (recursive)");
        System.out.println("[launcher] exec: " + String.join(" ", cmd));

        ProcessBuilder pb = new ProcessBuilder(cmd).directory(root.toFile());
        Process server = pb.start();

        // console: launcher stdin -> server stdin; server stdout+stderr -> logs + console
        Thread out = tee(server.getInputStream(), System.out, logs.resolve("server.log"));
        Thread err = tee(server.getErrorStream(), System.err, logs.resolve("server.log"));
        Thread stdin = forward(System.in, server.getOutputStream());

        int code = server.waitFor();
        stdin.interrupt();
        out.join(1000);
        err.join(1000);
        System.out.println("[launcher] server exited with code " + code);
        System.exit(code);
    }

    /** Pick the newest kernel jar in versions/ (deterministic, user-friendly). */
    static Path findKernel(Path versions) throws IOException {
        Files.createDirectories(versions);
        try (Stream<Path> s = Files.list(versions)) {
            List<Path> jars = s
                    .filter(p -> p.getFileName().toString().endsWith(".jar"))
                    .sorted(Comparator.comparing((Path p) -> p.getFileName().toString()).reversed())
                    .toList();
            if (jars.isEmpty()) {
                System.err.println("[launcher] no kernel jar in versions/ — drop e.g. purpur-1.21.10.jar there");
                System.exit(2);
            }
            return jars.get(0);
        }
    }

    static Path findAgent(Path root) {
        String os = System.getProperty("os.name", "").toLowerCase();
        String name = os.contains("win") ? "dist_agent.dll"
                : os.contains("mac") ? "libdist_agent.dylib"
                : "libdist_agent.so";
        Path agent = root.resolve(name);
        if (!Files.exists(agent)) {
            System.err.println("[launcher] native agent missing: " + agent);
            System.exit(2);
        }
        return agent;
    }

    static String javaBin() {
        String home = System.getProperty("java.home");
        String bin = File.separatorChar == '/' ? home + "/bin/java" : home + "\\bin\\java.exe";
        return bin;
    }

    /** JVM flags for the server process; override via DIST_JAVA_OPTS. */
    static List<String> javaOpts() {
        String env = System.getenv("DIST_JAVA_OPTS");
        if (env != null && !env.isBlank()) {
            return List.of(env.trim().split("\\s+"));
        }
        return List.of("-Xms512M", "-Xmx2G", "-XX:+UseG1GC", "-Dfile.encoding=UTF-8");
    }

    static Thread tee(InputStream in, OutputStream mirror, Path logFile) {
        Thread t = new Thread(() -> {
            try (var log = Files.newBufferedWriter(logFile, java.nio.charset.StandardCharsets.UTF_8,
                    java.nio.file.StandardOpenOption.CREATE, java.nio.file.StandardOpenOption.APPEND)) {
                byte[] buf = new byte[4096];
                int n;
                while ((n = in.read(buf)) != -1) {
                    mirror.write(buf, 0, n);
                    mirror.flush();
                    log.write(new String(buf, 0, n, java.nio.charset.StandardCharsets.UTF_8));
                    log.flush();
                }
            } catch (IOException ignored) {
            }
        }, "launcher-tee");
        t.setDaemon(true);
        t.start();
        return t;
    }

    static Thread forward(InputStream in, OutputStream out) {
        Thread t = new Thread(() -> {
            try {
                byte[] buf = new byte[4096];
                int n;
                while ((n = in.read(buf)) != -1) {
                    out.write(buf, 0, n);
                    out.flush();
                }
            } catch (IOException ignored) {
            }
        }, "launcher-stdin");
        t.setDaemon(true);
        t.start();
        return t;
    }
}
