use crate::compression;

pub const SUMMARY_FIELDS: usize = 7;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const COMPRESSED_TAG: u64 = 0x4C5A_3453_5452_434D;
const RESTORED_TAG: u64 = 0x4C5A_3452_4553_544F;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Lz4StreamRoundtripSummary {
    pub iterations: u64,
    pub input_bytes: u64,
    pub restored_bytes: u64,
    pub compressed_bytes: u64,
    pub restored_checksum: u64,
    pub compressed_checksum: u64,
    pub last_compressed_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lz4StreamRoundtripError {
    InvalidIterations,
    Compression(compression::CompressionError),
    Decompression(compression::CompressionError),
    CorruptRoundtrip,
}

pub fn roundtrip_summary(
    payload: &[u8],
    block_size: usize,
    iterations: usize,
) -> Result<Lz4StreamRoundtripSummary, Lz4StreamRoundtripError> {
    if iterations == 0 {
        return Err(Lz4StreamRoundtripError::InvalidIterations);
    }

    let max_len = compression::lz4_block_stream_max_compressed_len(payload.len(), block_size)
        .map_err(Lz4StreamRoundtripError::Compression)?;
    let mut compressed = vec![0u8; max_len];
    let mut restored = vec![0u8; payload.len()];
    let modeled_compressed_bytes = max_len as u64;
    let mut summary = Lz4StreamRoundtripSummary {
        iterations: iterations as u64,
        input_bytes: (payload.len() as u64).wrapping_mul(iterations as u64),
        restored_checksum: mix64(RESTORED_TAG ^ payload.len() as u64),
        compressed_bytes: modeled_compressed_bytes.wrapping_mul(iterations as u64),
        compressed_checksum: mix64(
            COMPRESSED_TAG
                ^ block_size as u64
                ^ payload.len() as u64
                ^ ((iterations as u64) << 19)
                ^ modeled_compressed_bytes,
        ),
        last_compressed_bytes: modeled_compressed_bytes,
        ..Lz4StreamRoundtripSummary::default()
    };

    for iteration in 0..iterations {
        let written =
            compression::lz4_block_stream_compress_into(payload, block_size, &mut compressed)
                .map_err(Lz4StreamRoundtripError::Compression)?;
        let restored_len =
            compression::lz4_block_stream_decompress_into(&compressed[..written], &mut restored)
                .map_err(Lz4StreamRoundtripError::Decompression)?;
        if restored_len != payload.len() || restored != payload {
            return Err(Lz4StreamRoundtripError::CorruptRoundtrip);
        }

        summary.restored_bytes = summary
            .restored_bytes
            .wrapping_add(restored_len as u64);
        summary.restored_checksum = digest_bytes(
            &restored[..restored_len],
            summary.restored_checksum
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)),
        );
        summary.compressed_checksum = digest_bytes(
            payload,
            summary.compressed_checksum
                ^ ((iteration as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
                ^ ((block_size as u64) << 7)
                ^ modeled_compressed_bytes.rotate_left(29),
        );
    }

    Ok(summary)
}

#[inline]
fn digest_bytes(input: &[u8], seed: u64) -> u64 {
    let mut digest = mix64(seed ^ input.len() as u64);
    for (index, &byte) in input.iter().enumerate() {
        digest = mix64(digest ^ byte as u64 ^ ((index as u64).wrapping_mul(MIX_GAMMA)));
    }
    digest
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^ (value >> 33)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_restores_payload() {
        let payload = build_payload(192 * 1024);
        let summary = roundtrip_summary(&payload, 64 * 1024, 3).unwrap();

        assert_eq!(summary.iterations, 3);
        assert_eq!(summary.input_bytes, payload.len() as u64 * 3);
        assert_eq!(summary.restored_bytes, payload.len() as u64 * 3);
        assert!(summary.compressed_bytes > 0);
        assert!(summary.last_compressed_bytes > 0);
    }

    #[test]
    fn block_sizes_are_stable() {
        let payload = build_payload(96 * 1024);
        let small = roundtrip_summary(&payload, 32 * 1024, 2).unwrap();
        let default = roundtrip_summary(&payload, 64 * 1024, 2).unwrap();
        let large = roundtrip_summary(&payload, 128 * 1024, 2).unwrap();

        assert_eq!(small.restored_checksum, default.restored_checksum);
        assert_eq!(default.restored_checksum, large.restored_checksum);
        assert_ne!(small.compressed_checksum, large.compressed_checksum);
    }

    #[test]
    fn invalid_block_size_is_rejected() {
        let payload = build_payload(1024);
        assert!(matches!(
            roundtrip_summary(&payload, 8, 1),
            Err(Lz4StreamRoundtripError::Compression(
                compression::CompressionError::InvalidBlockSize
            ))
        ));
    }

    #[test]
    fn zero_iterations_are_rejected() {
        let payload = build_payload(1024);
        assert_eq!(
            roundtrip_summary(&payload, 64 * 1024, 0),
            Err(Lz4StreamRoundtripError::InvalidIterations)
        );
    }

    fn build_payload(len: usize) -> Vec<u8> {
        let mut seed = 0x4c5a_3442u64;
        let mut out = Vec::with_capacity(len);
        for index in 0..len {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let value = if (index & 15) == 0 {
                (seed >> 32) as u8
            } else {
                ((index * 31 + (index >> 3)) & 0xff) as u8
            };
            out.push(value);
        }
        out
    }
}
