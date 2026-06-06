package dev.codex.libraryprobe;

import io.papermc.paper.plugin.loader.PluginClasspathBuilder;
import io.papermc.paper.plugin.loader.PluginLoader;
import io.papermc.paper.plugin.loader.library.impl.JarLibrary;
import java.nio.file.Path;

public final class LibraryProbeLoader implements PluginLoader {
    @Override
    public void classloader(final PluginClasspathBuilder classpathBuilder) {
        classpathBuilder.addLibrary(new JarLibrary(Path.of("library-probe-dep.jar")));
    }
}
