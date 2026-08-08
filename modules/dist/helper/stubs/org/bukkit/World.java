package org.bukkit;

/** Compile-time stub (NOT shipped). */
public interface World {
    void setChunkForceLoaded(int x, int z, boolean force);
    Chunk getChunkAt(int x, int z);
    int getMinHeight();
    int getMaxHeight();
}
