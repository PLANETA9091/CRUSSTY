package dev.crussty.hooks;

/**
 * Native bridge required by the storage-interception transform rules
 * (runtime/src/platform/storage.rs). The rule-injected ()V probes
 * (onChunkWrite/onChunkRead/onChunkWriteDone/onSaveStart/onSaveEnd/
 * onAutosave) delegate to the n* natives that the runtime registers
 * against the storage brick; the runtime defines this class in the system
 * class loader at agent start.
 */
public final class StorageHooks {
    private StorageHooks() {}

    /** Native surface backed by runtime/src/platform/storage.rs. */
    public static native boolean nStorageActive();
    public static native boolean nBeginSave();
    public static native boolean nEndSave();
    public static native void nChunkWritten();
    public static native void nMarkAutosave();
    public static native byte[] nReadChunk(int regionX, int regionZ, int chunkX, int chunkZ);
    public static native boolean nWriteChunk(int regionX, int regionZ, int chunkX, int chunkZ, byte[] payload);

    /** ()V probes injected by the storage transform rules. */
    public static void onChunkWrite() {
        nChunkWritten();
    }

    public static void onChunkRead() {
        // The adapter path needs explicit coordinates; module tooling that
        // captures them calls nReadChunk itself.
    }

    public static void onChunkWriteDone() {
    }

    public static void onSaveStart() {
        nBeginSave();
    }

    public static void onSaveEnd() {
        nEndSave();
    }

    public static void onAutosave() {
        nMarkAutosave();
    }
}