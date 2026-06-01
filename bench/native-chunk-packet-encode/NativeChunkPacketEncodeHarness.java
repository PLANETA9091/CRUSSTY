package net.minecraft.network.protocol.game;

import java.io.ByteArrayOutputStream;
import java.util.Arrays;
import java.util.BitSet;
import java.util.Locale;

public final class NativeChunkPacketEncodeHarness {
    private static final int LIGHT_BYTES = 2048;
    private static final int SECTIONS = 24;
    private static final int WARMUP = Integer.getInteger("native.chunkEncode.warmup", 2_000);
    private static final int ITERATIONS = Integer.getInteger("native.chunkEncode.iterations", 20_000);

    public static void main(String[] args) {
        LightFixture light = LightFixture.create();
        SectionFixture section = SectionFixture.create();

        assertSame("light", light.javaBytes(), light.nativeBytes());
        assertSame("section", section.javaBytes(), section.nativeBytes());

        long sink = 0L;
        for (int i = 0; i < WARMUP; i++) {
            sink ^= checksum(light.nativeBytes());
            sink ^= checksum(section.nativeBytes());
        }

        TimedRun javaLight = bestOf(light::javaBytes);
        TimedRun nativeLight = bestOf(light::nativeBytes);
        TimedRun javaSection = bestOf(section::javaBytes);
        TimedRun nativeSection = bestOf(section::nativeBytes);

        sink ^= javaLight.checksum ^ nativeLight.checksum ^ javaSection.checksum ^ nativeSection.checksum;
        print("light", javaLight, nativeLight);
        print("section", javaSection, nativeSection);
        System.out.println("digest=" + sink);
    }

    private static void assertSame(String label, byte[] expected, byte[] actual) {
        if (!Arrays.equals(expected, actual)) {
            throw new AssertionError(label + " parity mismatch expected=" + expected.length + " actual=" + actual.length);
        }
    }

    private static TimedRun bestOf(ByteProducer producer) {
        long bestNanos = Long.MAX_VALUE;
        long bestChecksum = 0L;
        int bestLength = 0;
        for (int round = 0; round < 8; round++) {
            long checksum = 0L;
            int length = 0;
            long start = System.nanoTime();
            for (int i = 0; i < ITERATIONS; i++) {
                byte[] bytes = producer.get();
                checksum ^= checksum(bytes);
                length += bytes.length;
            }
            long nanos = System.nanoTime() - start;
            if (nanos < bestNanos) {
                bestNanos = nanos;
                bestChecksum = checksum;
                bestLength = length;
            }
        }
        return new TimedRun(bestNanos, bestChecksum, bestLength);
    }

    private static void print(String label, TimedRun javaRun, TimedRun nativeRun) {
        System.out.printf(Locale.ROOT, "%s_java_best_ms=%.3f%n", label, javaRun.nanos / 1_000_000.0D);
        System.out.printf(Locale.ROOT, "%s_native_best_ms=%.3f%n", label, nativeRun.nanos / 1_000_000.0D);
        System.out.printf(Locale.ROOT, "%s_native_speedup_vs_java=%.3fx%n", label, (double) javaRun.nanos / (double) nativeRun.nanos);
        System.out.printf(Locale.ROOT, "%s_bytes_per_call=%d%n", label, javaRun.length / ITERATIONS);
    }

    private static void writeVarInt(ByteArrayOutputStream out, int value) {
        while ((value & -128) != 0) {
            out.write(value & 127 | 128);
            value >>>= 7;
        }
        out.write(value);
    }

    private static void writeLong(ByteArrayOutputStream out, long value) {
        for (int shift = 56; shift >= 0; shift -= 8) {
            out.write((int) (value >>> shift) & 255);
        }
    }

    private static void writeShort(ByteArrayOutputStream out, short value) {
        out.write((value >>> 8) & 255);
        out.write(value & 255);
    }

    private static void writeLongArray(ByteArrayOutputStream out, long[] values) {
        writeVarInt(out, values.length);
        for (long value : values) {
            writeLong(out, value);
        }
    }

    private static long checksum(byte[] bytes) {
        long hash = 0xcbf29ce484222325L;
        for (byte b : bytes) {
            hash = (hash ^ (b & 255)) * 0x100000001b3L;
        }
        return hash;
    }

    private interface ByteProducer {
        byte[] get();
    }

    private record TimedRun(long nanos, long checksum, int length) {
    }

    private record LightFixture(
        long[] skyMask,
        long[] blockMask,
        long[] emptySkyMask,
        long[] emptyBlockMask,
        byte[] skyUpdates,
        int skyUpdateCount,
        byte[] blockUpdates,
        int blockUpdateCount,
        byte[] expectedBytes
    ) {
        static LightFixture create() {
            byte[] skyUpdates = new byte[LIGHT_BYTES * 24];
            byte[] blockUpdates = new byte[LIGHT_BYTES * 18];
            Arrays.fill(skyUpdates, (byte) 0x5A);
            Arrays.fill(blockUpdates, (byte) 0xA5);
            LightFixture fixture = new LightFixture(bitsetLongs(0x00FF_FFFFL), bitsetLongs(0x0003_FFFFL), new long[0], new long[0], skyUpdates, 24, blockUpdates, 18, new byte[0]);
            return fixture.withExpectedBytes(fixture.javaBytes());
        }

        byte[] javaBytes() {
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            writeLongArray(out, this.skyMask);
            writeLongArray(out, this.blockMask);
            writeLongArray(out, this.emptySkyMask);
            writeLongArray(out, this.emptyBlockMask);
            writeUpdates(out, this.skyUpdates, this.skyUpdateCount);
            writeUpdates(out, this.blockUpdates, this.blockUpdateCount);
            return out.toByteArray();
        }

        LightFixture withExpectedBytes(byte[] expectedBytes) {
            return new LightFixture(
                this.skyMask,
                this.blockMask,
                this.emptySkyMask,
                this.emptyBlockMask,
                this.skyUpdates,
                this.skyUpdateCount,
                this.blockUpdates,
                this.blockUpdateCount,
                expectedBytes
            );
        }

        byte[] nativeBytes() {
            byte[] dst = new byte[this.expectedBytes.length];
            int written = PaperNativeChunkPacketEncode.nativeEncodeLightData(
                this.skyMask,
                this.blockMask,
                this.emptySkyMask,
                this.emptyBlockMask,
                this.skyUpdates,
                this.skyUpdateCount,
                this.blockUpdates,
                this.blockUpdateCount,
                dst
            );
            if (written < 0) {
                throw new AssertionError("native light encode failed: " + written);
            }
            return Arrays.copyOf(dst, written);
        }

        private static void writeUpdates(ByteArrayOutputStream out, byte[] updates, int count) {
            writeVarInt(out, count);
            for (int i = 0; i < count; i++) {
                writeVarInt(out, LIGHT_BYTES);
                out.write(updates, i * LIGHT_BYTES, LIGHT_BYTES);
            }
        }

        private static long[] bitsetLongs(long bits) {
            BitSet bitSet = BitSet.valueOf(new long[] { bits });
            return bitSet.toLongArray();
        }
    }

    private record SectionFixture(
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
        byte[] expectedBytes
    ) {
        static SectionFixture create() {
            short[] nonEmptyCounts = new short[SECTIONS];
            byte[] stateBits = new byte[SECTIONS];
            byte[] biomeBits = new byte[SECTIONS];
            int[] statePaletteOffsets = new int[SECTIONS + 1];
            int[] stateStorageOffsets = new int[SECTIONS + 1];
            int[] biomePaletteOffsets = new int[SECTIONS + 1];
            int[] biomeStorageOffsets = new int[SECTIONS + 1];
            ByteArrayOutputStream statePaletteBytes = new ByteArrayOutputStream();
            ByteArrayOutputStream biomePaletteBytes = new ByteArrayOutputStream();
            long[] stateStorageLongs = new long[SECTIONS * 256];
            long[] biomeStorageLongs = new long[SECTIONS];

            for (int section = 0; section < SECTIONS; section++) {
                nonEmptyCounts[section] = (short) (256 + section);
                stateBits[section] = 8;
                statePaletteBytes.write(3);
                statePaletteBytes.write(section);
                statePaletteBytes.write(1);
                statePaletteBytes.write(2);
                statePaletteOffsets[section + 1] = statePaletteBytes.size();
                stateStorageOffsets[section + 1] = stateStorageOffsets[section] + 256;
                for (int word = 0; word < 256; word++) {
                    stateStorageLongs[section * 256 + word] = ((long) section << 32) ^ word ^ 0x55AA55AAL;
                }

                biomeBits[section] = 2;
                biomePaletteBytes.write(1);
                biomePaletteBytes.write(section);
                biomePaletteOffsets[section + 1] = biomePaletteBytes.size();
                biomeStorageOffsets[section + 1] = biomeStorageOffsets[section] + 1;
                biomeStorageLongs[section] = section * 17L;
            }

            SectionFixture fixture = new SectionFixture(
                nonEmptyCounts,
                stateBits,
                statePaletteOffsets,
                statePaletteBytes.toByteArray(),
                stateStorageOffsets,
                stateStorageLongs,
                biomeBits,
                biomePaletteOffsets,
                biomePaletteBytes.toByteArray(),
                biomeStorageOffsets,
                biomeStorageLongs,
                new byte[0]
            );
            return fixture.withExpectedBytes(fixture.javaBytes());
        }

        byte[] javaBytes() {
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            for (int section = 0; section < this.nonEmptyCounts.length; section++) {
                writeShort(out, this.nonEmptyCounts[section]);
                out.write(this.stateBits[section]);
                out.write(this.statePaletteBytes, this.statePaletteOffsets[section], this.statePaletteOffsets[section + 1] - this.statePaletteOffsets[section]);
                for (int i = this.stateStorageOffsets[section]; i < this.stateStorageOffsets[section + 1]; i++) {
                    writeLong(out, this.stateStorageLongs[i]);
                }
                out.write(this.biomeBits[section]);
                out.write(this.biomePaletteBytes, this.biomePaletteOffsets[section], this.biomePaletteOffsets[section + 1] - this.biomePaletteOffsets[section]);
                for (int i = this.biomeStorageOffsets[section]; i < this.biomeStorageOffsets[section + 1]; i++) {
                    writeLong(out, this.biomeStorageLongs[i]);
                }
            }
            return out.toByteArray();
        }

        SectionFixture withExpectedBytes(byte[] expectedBytes) {
            return new SectionFixture(
                this.nonEmptyCounts,
                this.stateBits,
                this.statePaletteOffsets,
                this.statePaletteBytes,
                this.stateStorageOffsets,
                this.stateStorageLongs,
                this.biomeBits,
                this.biomePaletteOffsets,
                this.biomePaletteBytes,
                this.biomeStorageOffsets,
                this.biomeStorageLongs,
                expectedBytes
            );
        }

        byte[] nativeBytes() {
            byte[] dst = new byte[this.expectedBytes.length];
            int written = PaperNativeChunkPacketEncode.nativeEncodeSectionData(
                this.nonEmptyCounts,
                this.stateBits,
                this.statePaletteOffsets,
                this.statePaletteBytes,
                this.stateStorageOffsets,
                this.stateStorageLongs,
                this.biomeBits,
                this.biomePaletteOffsets,
                this.biomePaletteBytes,
                this.biomeStorageOffsets,
                this.biomeStorageLongs,
                dst
            );
            if (written < 0) {
                throw new AssertionError("native section encode failed: " + written);
            }
            return Arrays.copyOf(dst, written);
        }
    }
}
