use lz4::block as lz4_c_block;
use std::cell::RefCell;

pub const LZ4_BLOCK_STREAM_MAGIC: &[u8; 8] = b"LZ4Block";
pub const LZ4_BLOCK_STREAM_HEADER_LEN: usize = 21;
pub const LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE: usize = 64 * 1024;

const LZ4_BLOCK_STREAM_MIN_BLOCK_SIZE: usize = 64;
const LZ4_BLOCK_STREAM_MAX_BLOCK_SIZE: usize = 32 * 1024 * 1024;
const LZ4_BLOCK_STREAM_METHOD_RAW: u8 = 0x10;
const LZ4_BLOCK_STREAM_METHOD_LZ4: u8 = 0x20;
const LZ4_BLOCK_STREAM_DEFAULT_SEED: u32 = 0x9747_b28c;
const LZ4_BLOCK_STREAM_REUSABLE_SCRATCH_MAX_BLOCK_SIZE: usize = 1024 * 1024;

thread_local! {
    static LZ4_BLOCK_STREAM_SCRATCH: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionError {
    InvalidBlockSize,
    OutputTooSmall(usize),
    CorruptStream,
    UnexpectedEof,
    Lz4Compress,
    Lz4Decompress,
}

pub fn lz4_block_stream_max_compressed_len(
    input_len: usize,
    block_size: usize,
) -> Result<usize, CompressionError> {
    validate_block_size(block_size)?;
    let blocks = if input_len == 0 {
        0usize
    } else {
        (input_len + block_size - 1) / block_size
    };
    input_len
        .checked_add(
            (blocks + 1)
                .checked_mul(LZ4_BLOCK_STREAM_HEADER_LEN)
                .ok_or(CompressionError::OutputTooSmall(usize::MAX))?,
        )
        .ok_or(CompressionError::OutputTooSmall(usize::MAX))
}

pub fn lz4_block_stream_compress_into(
    input: &[u8],
    block_size: usize,
    dst: &mut [u8],
) -> Result<usize, CompressionError> {
    let compression_level = compression_level(block_size)?;
    let max_len = lz4_block_stream_max_compressed_len(input.len(), block_size)?;
    if dst.len() < max_len {
        return Err(CompressionError::OutputTooSmall(max_len));
    }

    let scratch_len =
        lz4_c_block::compress_bound(block_size).map_err(|_| CompressionError::Lz4Compress)?;
    if block_size <= LZ4_BLOCK_STREAM_REUSABLE_SCRATCH_MAX_BLOCK_SIZE {
        LZ4_BLOCK_STREAM_SCRATCH.with(|scratch| {
            let mut scratch = scratch.borrow_mut();
            if scratch.len() < scratch_len {
                scratch.resize(scratch_len, 0);
            }
            lz4_block_stream_compress_into_with_scratch(
                input,
                block_size,
                compression_level,
                dst,
                &mut scratch[..scratch_len],
            )
        })
    } else {
        let mut scratch = vec![0u8; scratch_len];
        lz4_block_stream_compress_into_with_scratch(
            input,
            block_size,
            compression_level,
            dst,
            &mut scratch,
        )
    }
}

fn lz4_block_stream_compress_into_with_scratch(
    input: &[u8],
    block_size: usize,
    compression_level: u8,
    dst: &mut [u8],
    scratch: &mut [u8],
) -> Result<usize, CompressionError> {
    let mut written = 0usize;

    for chunk in input.chunks(block_size) {
        let compressed_len = lz4_c_block::compress_to_buffer(chunk, None, false, scratch)
            .map_err(|_| CompressionError::Lz4Compress)?;
        let (method, payload) = if compressed_len < chunk.len() {
            (LZ4_BLOCK_STREAM_METHOD_LZ4, &scratch[..compressed_len])
        } else {
            (LZ4_BLOCK_STREAM_METHOD_RAW, chunk)
        };

        write_header(
            &mut dst[written..written + LZ4_BLOCK_STREAM_HEADER_LEN],
            method | compression_level,
            payload.len(),
            chunk.len(),
            checksum(chunk),
        );
        written += LZ4_BLOCK_STREAM_HEADER_LEN;
        dst[written..written + payload.len()].copy_from_slice(payload);
        written += payload.len();
    }

    write_header(
        &mut dst[written..written + LZ4_BLOCK_STREAM_HEADER_LEN],
        LZ4_BLOCK_STREAM_METHOD_RAW | compression_level,
        0,
        0,
        0,
    );
    written += LZ4_BLOCK_STREAM_HEADER_LEN;

    Ok(written)
}

#[cfg(test)]
fn reusable_scratch_capacity_for_tests() -> usize {
    LZ4_BLOCK_STREAM_SCRATCH.with(|scratch| scratch.borrow().capacity())
}

pub fn lz4_block_stream_decompress_into(
    input: &[u8],
    dst: &mut [u8],
) -> Result<usize, CompressionError> {
    let mut input_offset = 0usize;
    let mut output_offset = 0usize;

    loop {
        let header = read_header(input, input_offset)?;
        input_offset += LZ4_BLOCK_STREAM_HEADER_LEN;

        if header.original_len == 0 && header.compressed_len == 0 {
            if header.checksum != 0 {
                return Err(CompressionError::CorruptStream);
            }
            return Ok(output_offset);
        }

        let output_end = output_offset
            .checked_add(header.original_len)
            .ok_or(CompressionError::OutputTooSmall(usize::MAX))?;
        if output_end > dst.len() {
            return Err(CompressionError::OutputTooSmall(output_end));
        }

        let input_end = input_offset
            .checked_add(header.compressed_len)
            .ok_or(CompressionError::CorruptStream)?;
        if input_end > input.len() {
            return Err(CompressionError::UnexpectedEof);
        }

        let payload = &input[input_offset..input_end];
        let output = &mut dst[output_offset..output_end];

        match header.method {
            LZ4_BLOCK_STREAM_METHOD_RAW => {
                if header.compressed_len != header.original_len {
                    return Err(CompressionError::CorruptStream);
                }
                output.copy_from_slice(payload);
            }
            LZ4_BLOCK_STREAM_METHOD_LZ4 => {
                let decoded = lz4_c_block::decompress_to_buffer(
                    payload,
                    Some(header.original_len as i32),
                    output,
                )
                    .map_err(|_| CompressionError::Lz4Decompress)?;
                if decoded != header.original_len {
                    return Err(CompressionError::CorruptStream);
                }
            }
            _ => return Err(CompressionError::CorruptStream),
        }

        if checksum(output) != header.checksum {
            return Err(CompressionError::CorruptStream);
        }

        input_offset = input_end;
        output_offset = output_end;
    }
}

#[inline]
fn validate_block_size(block_size: usize) -> Result<(), CompressionError> {
    if (LZ4_BLOCK_STREAM_MIN_BLOCK_SIZE..=LZ4_BLOCK_STREAM_MAX_BLOCK_SIZE).contains(&block_size) {
        Ok(())
    } else {
        Err(CompressionError::InvalidBlockSize)
    }
}

#[inline]
fn compression_level(block_size: usize) -> Result<u8, CompressionError> {
    validate_block_size(block_size)?;
    let bits = usize::BITS - (block_size - 1).leading_zeros();
    Ok(bits.saturating_sub(10) as u8)
}

#[inline]
fn checksum(input: &[u8]) -> u32 {
    xxh32(input, LZ4_BLOCK_STREAM_DEFAULT_SEED) & 0x0FFF_FFFF
}

#[inline]
fn xxh32(input: &[u8], seed: u32) -> u32 {
    const PRIME1: u32 = 0x9E37_79B1;
    const PRIME2: u32 = 0x85EB_CA77;
    const PRIME3: u32 = 0xC2B2_AE3D;
    const PRIME4: u32 = 0x27D4_EB2F;
    const PRIME5: u32 = 0x1656_67B1;

    #[inline]
    fn round(acc: u32, lane: u32) -> u32 {
        const PRIME1: u32 = 0x9E37_79B1;
        const PRIME2: u32 = 0x85EB_CA77;
        let acc = acc.wrapping_add(lane.wrapping_mul(PRIME2));
        acc.rotate_left(13).wrapping_mul(PRIME1)
    }

    let mut index = 0usize;
    let len = input.len();
    let mut hash;

    if len >= 16 {
        let mut v1 = seed.wrapping_add(PRIME1).wrapping_add(PRIME2);
        let mut v2 = seed.wrapping_add(PRIME2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME1);

        while index <= len - 16 {
            v1 = round(v1, read_u32(input, index));
            index += 4;
            v2 = round(v2, read_u32(input, index));
            index += 4;
            v3 = round(v3, read_u32(input, index));
            index += 4;
            v4 = round(v4, read_u32(input, index));
            index += 4;
        }

        hash = v1
            .rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));
    } else {
        hash = seed.wrapping_add(PRIME5);
    }

    hash = hash.wrapping_add(len as u32);

    while index + 4 <= len {
        hash ^= read_u32(input, index).wrapping_mul(PRIME3);
        hash = hash.rotate_left(17).wrapping_mul(PRIME4);
        index += 4;
    }

    while index < len {
        hash ^= (input[index] as u32).wrapping_mul(PRIME5);
        hash = hash.rotate_left(11).wrapping_mul(PRIME1);
        index += 1;
    }

    hash ^= hash >> 15;
    hash = hash.wrapping_mul(PRIME2);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(PRIME3);
    hash ^= hash >> 16;
    hash
}

#[inline]
fn read_u32(input: &[u8], offset: usize) -> u32 {
    let bytes: [u8; 4] = input[offset..offset + 4]
        .try_into()
        .expect("slice must contain 4 bytes");
    u32::from_le_bytes(bytes)
}

#[inline]
fn write_header(dst: &mut [u8], token: u8, compressed_len: usize, original_len: usize, check: u32) {
    dst[..LZ4_BLOCK_STREAM_MAGIC.len()].copy_from_slice(LZ4_BLOCK_STREAM_MAGIC);
    dst[LZ4_BLOCK_STREAM_MAGIC.len()] = token;
    write_i32_le(compressed_len as i32, dst, LZ4_BLOCK_STREAM_MAGIC.len() + 1);
    write_i32_le(original_len as i32, dst, LZ4_BLOCK_STREAM_MAGIC.len() + 5);
    write_i32_le(check as i32, dst, LZ4_BLOCK_STREAM_MAGIC.len() + 9);
}

#[inline]
fn write_i32_le(value: i32, dst: &mut [u8], offset: usize) {
    dst[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_header(input: &[u8], offset: usize) -> Result<BlockHeader, CompressionError> {
    let header_end = offset
        .checked_add(LZ4_BLOCK_STREAM_HEADER_LEN)
        .ok_or(CompressionError::CorruptStream)?;
    if header_end > input.len() {
        return Err(CompressionError::UnexpectedEof);
    }

    if &input[offset..offset + LZ4_BLOCK_STREAM_MAGIC.len()] != LZ4_BLOCK_STREAM_MAGIC {
        return Err(CompressionError::CorruptStream);
    }

    let token = input[offset + LZ4_BLOCK_STREAM_MAGIC.len()];
    let method = token & 0xF0;
    let max_original_len = 1usize << (10 + (token & 0x0F) as usize);
    let compressed_len = read_len(input, offset + LZ4_BLOCK_STREAM_MAGIC.len() + 1)?;
    let original_len = read_len(input, offset + LZ4_BLOCK_STREAM_MAGIC.len() + 5)?;
    let check = read_i32_le(input, offset + LZ4_BLOCK_STREAM_MAGIC.len() + 9)? as u32;

    if method != LZ4_BLOCK_STREAM_METHOD_RAW && method != LZ4_BLOCK_STREAM_METHOD_LZ4 {
        return Err(CompressionError::CorruptStream);
    }
    if original_len > max_original_len {
        return Err(CompressionError::CorruptStream);
    }
    if (original_len == 0) != (compressed_len == 0) {
        return Err(CompressionError::CorruptStream);
    }
    if method == LZ4_BLOCK_STREAM_METHOD_RAW && original_len != compressed_len {
        return Err(CompressionError::CorruptStream);
    }

    Ok(BlockHeader {
        method,
        compressed_len,
        original_len,
        checksum: check,
    })
}

#[inline]
fn read_len(input: &[u8], offset: usize) -> Result<usize, CompressionError> {
    let value = read_i32_le(input, offset)?;
    if value < 0 {
        Err(CompressionError::CorruptStream)
    } else {
        Ok(value as usize)
    }
}

#[inline]
fn read_i32_le(input: &[u8], offset: usize) -> Result<i32, CompressionError> {
    let bytes: [u8; 4] = input[offset..offset + 4]
        .try_into()
        .map_err(|_| CompressionError::UnexpectedEof)?;
    Ok(i32::from_le_bytes(bytes))
}

#[derive(Debug, Clone, Copy)]
struct BlockHeader {
    method: u8,
    compressed_len: usize,
    original_len: usize,
    checksum: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compress_to_vec(input: &[u8], block_size: usize) -> Vec<u8> {
        let max_len = lz4_block_stream_max_compressed_len(input.len(), block_size).unwrap();
        let mut compressed = vec![0u8; max_len];
        let written = lz4_block_stream_compress_into(input, block_size, &mut compressed).unwrap();
        compressed.truncate(written);
        compressed
    }

    #[test]
    fn compression_level_matches_java_lz4_block_stream() {
        assert_eq!(compression_level(64), Ok(0));
        assert_eq!(compression_level(1024), Ok(0));
        assert_eq!(
            compression_level(LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE),
            Ok(6)
        );
        assert_eq!(compression_level(128 * 1024), Ok(7));
    }

    #[test]
    fn max_compressed_len_includes_each_block_header_and_eof() {
        assert_eq!(
            lz4_block_stream_max_compressed_len(0, LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE),
            Ok(LZ4_BLOCK_STREAM_HEADER_LEN)
        );
        assert_eq!(
            lz4_block_stream_max_compressed_len(96 * 1024, LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE),
            Ok((96 * 1024) + (3 * LZ4_BLOCK_STREAM_HEADER_LEN))
        );
    }

    #[test]
    fn empty_stream_is_single_eof_block() {
        let mut compressed = [0u8; LZ4_BLOCK_STREAM_HEADER_LEN];
        let written = lz4_block_stream_compress_into(
            &[],
            LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE,
            &mut compressed,
        )
        .unwrap();
        assert_eq!(written, LZ4_BLOCK_STREAM_HEADER_LEN);
        assert_eq!(&compressed[..LZ4_BLOCK_STREAM_MAGIC.len()], LZ4_BLOCK_STREAM_MAGIC);
        assert_eq!(
            compressed[LZ4_BLOCK_STREAM_MAGIC.len()],
            LZ4_BLOCK_STREAM_METHOD_RAW | 6
        );

        let mut restored = [];
        assert_eq!(
            lz4_block_stream_decompress_into(&compressed, &mut restored),
            Ok(0)
        );
    }

    #[test]
    fn checksum_matches_lz4_java_checksum_adapter_mask() {
        let block = vec![7u8; LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE];
        assert_eq!(checksum(&block), 0x00CF_5907);
    }

    #[test]
    fn block_stream_round_trips_repetitive_and_noisy_data() {
        let mut input = vec![0u8; 128 * 1024 + 17];
        for (index, value) in input.iter_mut().enumerate() {
            *value = if index % 29 == 0 {
                ((index * 131) & 0xFF) as u8
            } else {
                ((index >> 5) & 0xFF) as u8
            };
        }

        let max_len = lz4_block_stream_max_compressed_len(
            input.len(),
            LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE,
        )
        .unwrap();
        let mut compressed = vec![0u8; max_len];
        let written = lz4_block_stream_compress_into(
            &input,
            LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE,
            &mut compressed,
        )
        .unwrap();

        let mut restored = vec![0u8; input.len()];
        let restored_len = lz4_block_stream_decompress_into(
            &compressed[..written],
            &mut restored,
        )
        .unwrap();
        assert_eq!(restored_len, input.len());
        assert_eq!(restored, input);
    }

    #[test]
    fn block_stream_output_is_stable_across_reused_scratch() {
        let mut input = vec![0u8; 96 * 1024 + 333];
        for (index, value) in input.iter_mut().enumerate() {
            *value = (index.wrapping_mul(37).wrapping_add(index >> 9) & 0xFF) as u8;
        }

        let expected = compress_to_vec(&input, LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE);

        let mut unrelated = vec![0u8; 256 * 1024 + 11];
        for (index, value) in unrelated.iter_mut().enumerate() {
            *value = (index.wrapping_mul(97).wrapping_add(index >> 5) & 0xFF) as u8;
        }
        let _ = compress_to_vec(&unrelated, 256 * 1024);

        let actual = compress_to_vec(&input, LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE);
        assert_eq!(actual, expected);

        let mut restored = vec![0u8; input.len()];
        let restored_len = lz4_block_stream_decompress_into(&actual, &mut restored).unwrap();
        assert_eq!(restored_len, input.len());
        assert_eq!(restored, input);
    }

    #[test]
    fn oversized_block_size_does_not_grow_reusable_scratch() {
        let ordinary = vec![7u8; 4096];
        let _ = compress_to_vec(&ordinary, LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE);
        let retained_capacity = reusable_scratch_capacity_for_tests();

        let oversized_block_size =
            LZ4_BLOCK_STREAM_REUSABLE_SCRATCH_MAX_BLOCK_SIZE + LZ4_BLOCK_STREAM_MIN_BLOCK_SIZE;
        let input = vec![42u8; 8192];
        let compressed = compress_to_vec(&input, oversized_block_size);

        assert_eq!(reusable_scratch_capacity_for_tests(), retained_capacity);

        let mut restored = vec![0u8; input.len()];
        let restored_len = lz4_block_stream_decompress_into(&compressed, &mut restored).unwrap();
        assert_eq!(restored_len, input.len());
        assert_eq!(restored, input);
    }

    #[test]
    fn block_stream_uses_lz4_method_when_smaller() {
        let input = vec![7u8; 96 * 1024];
        let max_len = lz4_block_stream_max_compressed_len(
            input.len(),
            LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE,
        )
        .unwrap();
        let mut compressed = vec![0u8; max_len];
        let written = lz4_block_stream_compress_into(
            &input,
            LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE,
            &mut compressed,
        )
        .unwrap();
        assert!(written < input.len());
        assert_eq!(
            compressed[LZ4_BLOCK_STREAM_MAGIC.len()] & 0xF0,
            LZ4_BLOCK_STREAM_METHOD_LZ4
        );

        let mut restored = vec![0u8; input.len()];
        let restored_len = lz4_block_stream_decompress_into(
            &compressed[..written],
            &mut restored,
        )
        .unwrap();
        assert_eq!(restored_len, input.len());
        assert_eq!(restored, input);
    }

    #[test]
    fn decompression_reports_required_output_size() {
        let input = vec![42u8; 4096];
        let mut compressed = vec![
            0u8;
            lz4_block_stream_max_compressed_len(
                input.len(),
                LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE,
            )
            .unwrap()
        ];
        let written = lz4_block_stream_compress_into(
            &input,
            LZ4_BLOCK_STREAM_DEFAULT_BLOCK_SIZE,
            &mut compressed,
        )
        .unwrap();

        let mut too_small = vec![0u8; input.len() - 1];
        assert_eq!(
            lz4_block_stream_decompress_into(&compressed[..written], &mut too_small),
            Err(CompressionError::OutputTooSmall(input.len()))
        );
    }
}
