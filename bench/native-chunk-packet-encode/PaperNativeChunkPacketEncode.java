package net.minecraft.network.protocol.game;

final class PaperNativeChunkPacketEncode {
    static {
        System.loadLibrary("paper_native_chunk_encode_jni");
    }

    private PaperNativeChunkPacketEncode() {
    }

    static native int nativeEncodeLightData(
        long[] skyYMaskLongs,
        long[] blockYMaskLongs,
        long[] emptySkyYMaskLongs,
        long[] emptyBlockYMaskLongs,
        byte[] skyUpdates,
        int skyUpdateCount,
        byte[] blockUpdates,
        int blockUpdateCount,
        byte[] dst
    );

    static native int nativeEncodeSectionData(
        short[] nonEmptyCounts,
        byte[] stateBits,
        int[] statePaletteOffsets,
        byte[] statePaletteBytes,
        int[] stateStorageOffsets,
        long[] stateStorageLongs,
        byte[] biomeBits,
        int[] biomePaletteOffsets,
        byte[] biomePaletteBytes,
        int[] biomeStorageOffsets,
        long[] biomeStorageLongs,
        byte[] dst
    );
}
