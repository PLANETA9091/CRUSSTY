package dev.dist;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.List;
import org.bukkit.Bukkit;
import org.bukkit.Chunk;
import org.bukkit.ChunkSnapshot;
import org.bukkit.World;

/**
 * The kernel-facing half of the dist engine, ported from v1 dist-paper
 * (RegionManager + RegionHasher). Lives in the kernel's class loader (defined
 * via DefineClass, like the area_map ops helpers) so it can use Bukkit API
 * directly; the Rust module calls these two static methods by reflection:
 *
 *   1. {@link #forceChunks(int, int, int, boolean)} — force-load (or release)
 *      the chunk square of a dist region, so its simulation runs on this node.
 *   2. {@link #hashRegion(int, int, int)} — SHA-256 over the serialized block
 *      data of the region's loaded chunks; the oracle compares these hashes to
 *      detect divergence (same algorithm as v1 RegionHasher).
 *
 * Metrics (load) are read by the Rust side from
 * MinecraftServer.getServer().getTickTimesNanos().
 */
public final class DistKernel {

    private DistKernel() {}

    /** Chunk coordinates covered by this region (region 0 at 0,0; grows +x). */
    private static List<int[]> chunkCoords(int regionId, int chunksPerSide, int regionsPerRow) {
        int row = regionId / regionsPerRow;
        int col = regionId % regionsPerRow;
        int baseX = col * chunksPerSide;
        int baseZ = row * chunksPerSide;
        List<int[]> out = new ArrayList<>(chunksPerSide * chunksPerSide);
        for (int x = 0; x < chunksPerSide; x++) {
            for (int z = 0; z < chunksPerSide; z++) {
                out.add(new int[] {baseX + x, baseZ + z});
            }
        }
        return out;
    }

    /** Force-load or release all chunks of a region in the overworld. */
    public static void forceChunks(int regionId, int chunksPerSide, int regionsPerRow, boolean force) {
        World world = Bukkit.getWorld("world");
        if (world == null) {
            throw new IllegalStateException("world 'world' not loaded");
        }
        for (int[] c : chunkCoords(regionId, chunksPerSide, regionsPerRow)) {
            world.setChunkForceLoaded(c[0], c[1], force);
        }
    }

    /**
     * State-hash of an owned region: SHA-256 over the block data of its loaded
     * chunks. The oracle compares these hashes to detect divergence.
     */
    public static byte[] hashRegion(int regionId, int chunksPerSide, int regionsPerRow) {
        World world = Bukkit.getWorld("world");
        if (world == null) {
            throw new IllegalStateException("world 'world' not loaded");
        }
        MessageDigest sha;
        try {
            sha = MessageDigest.getInstance("SHA-256");
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException(e);
        }
        sha.update("region".getBytes(StandardCharsets.UTF_8));
        sha.update(intToBytes(regionId));
        for (int[] c : chunkCoords(regionId, chunksPerSide, regionsPerRow)) {
            Chunk chunk = world.getChunkAt(c[0], c[1]);
            if (!chunk.isLoaded()) {
                continue;
            }
            ChunkSnapshot cs = chunk.getChunkSnapshot(true, false, false);
            sha.update(intToBytes(cs.getX()));
            sha.update(intToBytes(cs.getZ()));
            for (int y = world.getMinHeight(); y < world.getMaxHeight(); y++) {
                for (int x = 0; x < 16; x++) {
                    for (int z = 0; z < 16; z++) {
                        sha.update(cs.getBlockData(x, y, z).getAsString().getBytes(StandardCharsets.UTF_8));
                    }
                }
            }
        }
        return sha.digest();
    }

    private static byte[] intToBytes(int v) {
        return new byte[] {
            (byte) (v >>> 24), (byte) (v >>> 16), (byte) (v >>> 8), (byte) v,
        };
    }
}
