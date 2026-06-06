package dev.codex.libraryprobe;

import dev.codex.libraryprobe.lib.LibraryProbeDependency;
import org.bukkit.plugin.java.JavaPlugin;

public final class LibraryProbePlugin extends JavaPlugin {
    @Override
    public void onEnable() {
        getLogger().info("LIBRARY_PROBE lifecycle=enable");
        getLogger().info("LIBRARY_PROBE dependency=" + LibraryProbeDependency.message());
    }

    @Override
    public void onDisable() {
        getLogger().info("LIBRARY_PROBE lifecycle=disable");
    }
}
