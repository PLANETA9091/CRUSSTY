package org.bukkit;

/** Compile-time stub (NOT shipped). */
public interface Chunk {
    boolean isLoaded();
    ChunkSnapshot getChunkSnapshot(boolean includeMaxblocky, boolean includeBiome, boolean includeBiomeTempRain);
}
