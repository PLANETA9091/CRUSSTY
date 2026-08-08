# modules/crussty/native — published Crussty CE native libraries

This directory contains the binary files the `crussty` module loads at runtime
(the module is a dead injector without them) and the manifest that is the
single source of truth for its JNI bridge table.

| File | Purpose |
|---|---|
| `libpaper_native_jni.so` | 280 JNI exports (area map, ore/ticket/waypoint benchmarks, improved noise, climate, chunk wrap, LZ4 stream, ...) |
| `libpaper_native_chunk_encode_jni.so` | 3 JNI exports (chunk packet encode) |
| `JNI_EXPORTS.manifest` | 283 `class\|method\|sig\|symbol` rows — source of truth for `modules/crussty/src/jni_table.rs` |
| `LICENSE` | MIT, see below |

## Provenance & license

The libraries were built (September 2025, release profile, linux x86-64) from
the Crussty CE Rust crates `paper-native-jni` and
`paper-native-chunk-encode-jni` (the "Crustsy CE native surface" — 283
`Java_*` JNI exports). The Java bridge classes are not part of that project;
the `crussty` module re-declares them here (see `modules/crussty/src/lib.rs`).

License: MIT — Copyright (c) 2025 ANDMC / P500 Project Contributors (see
`LICENSE`). Same license as this repository's root `LICENSE`.

SHA-256:

```
d8f821aa6dd3962723899085ee31f89a939822e7d8bb98833ed6f32e136c4f85  libpaper_native_jni.so
713977af51dd60ca83220cfa793c22117490b08baa8e0511f88b0740785709d2  libpaper_native_chunk_encode_jni.so
```

## How the bridge table is regenerated (single source of truth)

Never edit `modules/crussty/src/jni_table.rs` by hand. Edit
`JNI_EXPORTS.manifest` (or regenerate it from a newer jni_table with `dump`),
then:

```bash
python3 scripts/gen_crussty_table.py dump        # jni_table.rs -> manifest (migration only)
python3 scripts/gen_crussty_table.py render      # manifest -> jni_table.rs
python3 scripts/gen_crussty_table.py render --check   # CI: fail if out of sync
python3 scripts/gen_crussty_table.py verify      # cross-check manifest vs shipped .so (nm -D)
```

Checked-in invariant (CI): regenerated `jni_table.rs` must byte-match the
committed file, and (when binaries are present) every manifest symbol must
exist in the shipped libraries.

## Runtime wiring

`modules/crussty/src/lib.rs` looks for the libraries in `<plugin>/native/`
(or the plugin dir): `libpaper_native_jni.so` is required — the module logs
`missing …` and skips injection; the chunk-encode lib is optional.

`scripts/build-single-jar.sh` embeds this `native/` subdir into the
single-jar distribution automatically.

## Rebuilding the .so files

The crates themselves live upstream in Crussty CE (closed source); the table
in this repo does not require them — only the binaries above. If you have the
crates, build with:

```sh
cargo build --release -p paper-native-jni -p paper-native-chunk-encode-jni \
  --manifest-path <crussty-ce>/native/Cargo.toml
cp native/target/release/libpaper_native_jni.so modules/crussty/native/
cp native/target/release/libpaper_native_chunk_encode_jni.so modules/crussty/native/
python3 scripts/gen_crussty_table.py verify
```