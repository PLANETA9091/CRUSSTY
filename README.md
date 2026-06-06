<p align="center">
  <img src="https://img.shields.io/badge/Minecraft-1.21.10-brightgreen" alt="MC Version">
  <img src="https://img.shields.io/badge/PaperMC-Paper%20Build%20130-orange" alt="Paper">
  <img src="https://img.shields.io/badge/Rust-1.7x%2B-dea584" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
</p>

<h1 align="center">Paper Native Accelerator — Community Edition</h1>

<p align="center">
  <strong>Rust-powered hot-path replacement for PaperMC via JNI</strong><br>
  Up to <strong>18,000x speedup</strong> on targeted operations | 80+ native modules | Zero plugin changes
</p>

---

## What Is This?

Paper Native Accelerator replaces performance-critical Java code paths in PaperMC with
Rust implementations called via JNI (Java Native Interface). This is a **drop-in
acceleration layer** — your existing Paper plugins, worlds, and configurations work
unchanged.

The project was developed as part of the P500 initiative: pushing a single PaperMC
server to handle **500+ concurrent players** at 32/32 view/simulation distance with
full production plugin load.

This is the **Community Edition (CE)** — the complete source code for the Rust native
modules, runtime launcher, benchmark suite, and documentation. The community is
invited to fork, extend, and improve.

## Architecture

The system has 4 Rust crates in a Cargo workspace:

- **paper-native-core** — Pure Rust parity implementations of Java hot paths
- **paper-native-jni** — JNI bridge (cdylib) exposing native functions to the JVM
- **paper-native-chunk-encode-core** — Chunk packet section/light encoding in Rust
- **paper-native-chunk-encode-jni** — JNI bridge for chunk encoding

Compiled `.so` libraries are loaded at runtime. Each module can be toggled via
environment variables without recompiling Paper.

## Modules and Speedups

| Category | Modules | Peak Speedup |
|----------|---------|-------------|
| Waypoint system | snapshot, table_view, manager_skip (8 shapes) | 4,500 - 28,000x |
| Placed feature traversal | ore_feature_loop, placed_feature_traversal | 21 - 30x |
| Jigsaw structures | jigsaw_canAttach | 31x |
| Chunk heightmaps | levelchunk_heightmap, protochunk_heightmap | 7.6x |
| Climate / Biome | climate_rtree, distance, parameter (4 modules) | 3 - 5x |
| Surface rules | sequence_array (2 modules) | 1.9 - 6.3x |
| Ticket system | search, compare, pack, chunk_stage | 3.2 - 3.6x |
| Area map | area_map_update | 1.3x |
| Noise generation | improved_noise, normal_noise, perlin_noise, blended_noise | varies |
| Chunk encoding | section + light encode (separate .so) | significant |
| Compression | LZ4 + XXH32 roundtrip | fast |
| Plugin loading | dependency, classloader, directory_scan, startup_rollup | moderate |
| Entity tracking | bounding_box, lookup_status, nearby_player_map | moderate |
| NBT / Hashing | nbt (3 modules), SHA-256, varint, position | fast |

See `native/README.md` for full module-by-module documentation.

## Runtime Hooks (Toggleable)

| Env Variable | Module | Default |
|---|---|---|
| `PAPER_NATIVE_IMPROVED_NOISE` | Worldgen improved noise | enabled |
| `PAPER_NATIVE_NORMAL_NOISE` | Normal noise batching | enabled |
| `PAPER_NATIVE_PERLIN_NOISE` | Perlin noise | disabled (regression) |
| `PAPER_NATIVE_CLIMATE_RTREE` | Biome climate R-tree search | enabled |
| `PAPER_NATIVE_AREA_MAP` | Chunk area map updates | enabled |
| `PAPER_NATIVE_CHUNK_PACKET_ENCODE` | Chunk packet encoding | enabled |

## Project Structure

```
paper-native-accelerator/
├── native/                  # Rust workspace (4 crates, 98 .rs files)
│   ├── paper-native-core/   # Pure Rust implementations
│   ├── paper-native-jni/    # JNI bridge (cdylib)
│   ├── paper-native-chunk-encode-core/
│   ├── paper-native-chunk-encode-jni/
│   ├── Cargo.toml
│   └── README.md            # Detailed module docs
├── scripts/                 # Benchmark and automation (340 files)
│   ├── mc_bot_swarm.cjs     # Node.js bot swarm (P100/P500)
│   ├── mc_bot_swarm.sh
│   ├── p100.sh / p500.sh    # Benchmark runners
│   └── *.py                 # Python validation gates
├── docs/                    # Documentation (44 files)
│   ├── ROADMAP.md
│   ├── STATE.md
│   ├── TESTING.md
│   └── PARITY_MATRIX.md
├── build/                   # Build helpers
├── test-plugins/            # Custom Java test plugins
├── artifacts/
│   └── run.sh               # Runtime launcher
├── .gitignore
├── LICENSE
└── README.md
```

## Building from Source

### Prerequisites

- **Rust** 1.75+ (rustup recommended)
- **Java** 21+ (for PaperMC)
- **C toolchain** (gcc, make — for JNI compilation)
- **Linux** x86_64 (tested on Debian 12 / Ubuntu 22.04)

### Build Native Libraries

```bash
cd native
cargo build --release

# Output:
# target/release/libpaper_native_jni.so
# target/release/libpaper_native_chunk_encode_jni.so
```

### Build Patched Paper

Clone PaperMC 1.21.10 and apply patches:

```bash
git clone https://github.com/PaperMC/Paper.git --branch ver/1.21.10 --depth 1 paper-source
# Apply patches from this project
```

### Run with Native Acceleration

```bash
export LD_LIBRARY_PATH=/path/to/native/target/release
export PAPER_NATIVE_IMPROVED_NOISE=1
export PAPER_NATIVE_CLIMATE_RTREE=1
export PAPER_NATIVE_CHUNK_PACKET_ENCODE=1

java -Xmx24G \
  -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 \
  -Djava.library.path=/path/to/native/target/release \
  -jar paper.jar nogui
```

## Benchmarking

```bash
# P100 - 100 bots (quick test)
cd scripts
bash p100.sh 120

# P500 - 500 bots (full stress test, 10 shards x 50 bots)
bash p500.sh 300

# Custom
bash p100.sh 60 127.0.0.1 25567 move
```

## Community Edition - What's Included

### Included
- Complete Rust source for all 80+ native modules
- JNI bridge code (13,000+ lines)
- Runtime launcher (run.sh) with hook toggles
- Full benchmark suite (340 scripts)
- Architecture and module documentation
- Build system (Cargo workspace, 4 crates)

### Not Included (build yourself or get from releases)
- Pre-compiled .so binaries (platform-specific)
- Pre-patched Paper JAR
- AppCDS archive (JVM-specific)
- Upstream Paper source (clone separately)

### Known Limitations
- Perlin noise native module regressed vs Java (disabled by default)
- Beardifier bury and YClamped gradient are faster in Java
- Chunk expire count is slower in native (left as Java)
- JNI call overhead (~100ns) can negate gains for very small operations

## Contributing

Contributions are welcome! Areas where help is especially needed:

**Good First Issues:**
- Profile PaperMC to find new hot paths not yet covered
- Add SIMD intrinsics for noise generation or compression
- Windows support (.dll builds)
- AARCH64/ARM support

**How to Contribute:**
1. Fork this repository
2. Create a feature branch
3. Write Rust implementation in `paper-native-core/src/`
4. Add JNI bridge methods in `paper-native-jni/src/lib.rs`
5. Benchmark against Java baseline using `scripts/`
6. Submit a Pull Request with benchmarks

## Acknowledgments

- **PaperMC team** for Paper server and the Moonrise chunk system
- **Spottedleaf** for Starlight lighting engine
- **jni-rs** crate for the Rust-JNI bridge library

## License

MIT License - see [LICENSE](LICENSE) file.
