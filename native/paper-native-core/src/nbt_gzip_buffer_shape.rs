use std::convert::TryFrom;

pub const SUMMARY_FIELDS: usize = 8;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const SUMMARY_TAG: u64 = 0x4E42_5447_5A42_5546;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NbtGzipBufferShapeSummary {
    pub write_calls: u64,
    pub input_bytes: u64,
    pub outer_flushes: u64,
    pub gzip_input_calls: u64,
    pub direct_writes: u64,
    pub modeled_gzip_chunks: u64,
    pub largest_write: u64,
    pub checksum: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NbtGzipBufferShapeError {
    InvalidWriteLength,
    InvalidRepeats,
    InvalidBufferSize,
}

pub fn shape_summary(
    write_lengths: &[i32],
    repeats: usize,
    outer_buffer_size: usize,
    gzip_buffer_size: usize,
) -> Result<NbtGzipBufferShapeSummary, NbtGzipBufferShapeError> {
    if repeats == 0 {
        return Err(NbtGzipBufferShapeError::InvalidRepeats);
    }
    if outer_buffer_size == 0 || gzip_buffer_size == 0 {
        return Err(NbtGzipBufferShapeError::InvalidBufferSize);
    }

    let mut summary = NbtGzipBufferShapeSummary {
        checksum: mix64(
            SUMMARY_TAG
                ^ write_lengths.len() as u64
                ^ ((repeats as u64) << 11)
                ^ ((outer_buffer_size as u64) << 23)
                ^ ((gzip_buffer_size as u64) << 37),
        ),
        ..NbtGzipBufferShapeSummary::default()
    };

    for repeat in 0..repeats {
        let mut outer_used = 0usize;
        for (index, &length) in write_lengths.iter().enumerate() {
            let length = usize::try_from(length)
                .map_err(|_| NbtGzipBufferShapeError::InvalidWriteLength)?;
            summary.write_calls = summary.write_calls.wrapping_add(1);
            summary.input_bytes = summary.input_bytes.wrapping_add(length as u64);
            summary.largest_write = summary.largest_write.max(length as u64);
            summary.checksum = mix64(
                summary.checksum
                    ^ ((repeat as u64).wrapping_mul(MIX_GAMMA))
                    ^ ((index as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
                    ^ ((length as u64) << 5)
                    ^ ((outer_used as u64) << 19),
            );

            if length == 0 {
                continue;
            }

            if length >= outer_buffer_size {
                flush_outer(
                    &mut summary,
                    &mut outer_used,
                    gzip_buffer_size,
                    repeat,
                    index,
                );
                summary.direct_writes = summary.direct_writes.wrapping_add(1);
                gzip_receive(&mut summary, length, gzip_buffer_size, repeat, index);
                continue;
            }

            if length > outer_buffer_size - outer_used {
                flush_outer(
                    &mut summary,
                    &mut outer_used,
                    gzip_buffer_size,
                    repeat,
                    index,
                );
            }
            outer_used += length;
        }

        flush_outer(
            &mut summary,
            &mut outer_used,
            gzip_buffer_size,
            repeat,
            write_lengths.len(),
        );
    }

    Ok(summary)
}

fn flush_outer(
    summary: &mut NbtGzipBufferShapeSummary,
    outer_used: &mut usize,
    gzip_buffer_size: usize,
    repeat: usize,
    index: usize,
) {
    if *outer_used == 0 {
        return;
    }
    let flushed = *outer_used;
    *outer_used = 0;
    summary.outer_flushes = summary.outer_flushes.wrapping_add(1);
    gzip_receive(summary, flushed, gzip_buffer_size, repeat, index);
}

fn gzip_receive(
    summary: &mut NbtGzipBufferShapeSummary,
    input_len: usize,
    gzip_buffer_size: usize,
    repeat: usize,
    index: usize,
) {
    summary.gzip_input_calls = summary.gzip_input_calls.wrapping_add(1);
    let chunks = input_len.div_ceil(gzip_buffer_size).max(1);
    summary.modeled_gzip_chunks = summary.modeled_gzip_chunks.wrapping_add(chunks as u64);
    summary.checksum = mix64(
        summary.checksum
            ^ ((input_len as u64) << 3)
            ^ ((gzip_buffer_size as u64) << 17)
            ^ ((chunks as u64) << 41)
            ^ ((repeat as u64).wrapping_mul(0xD6E8_FD9A_2C4F_6B1D))
            ^ ((index as u64).wrapping_mul(MIX_GAMMA)),
    );
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
    fn larger_outer_buffer_reduces_flushes() {
        let writes = [1, 2, 64, 4096, 12_000, 17, 21_000];
        let current = shape_summary(&writes, 4, 8 * 1024, 512).unwrap();
        let prebuffer = shape_summary(&writes, 4, 64 * 1024, 512).unwrap();

        assert_eq!(current.write_calls, prebuffer.write_calls);
        assert_eq!(current.input_bytes, prebuffer.input_bytes);
        assert!(prebuffer.outer_flushes < current.outer_flushes);
    }

    #[test]
    fn larger_gzip_buffer_reduces_modeled_chunks() {
        let writes = [32_000, 4_096, 71_000, 128];
        let current = shape_summary(&writes, 3, 8 * 1024, 512).unwrap();
        let gzip64k = shape_summary(&writes, 3, 8 * 1024, 64 * 1024).unwrap();

        assert_eq!(current.input_bytes, gzip64k.input_bytes);
        assert!(gzip64k.modeled_gzip_chunks < current.modeled_gzip_chunks);
    }

    #[test]
    fn direct_writes_track_large_writes() {
        let writes = [128, 16_384, 256, 32_768];
        let summary = shape_summary(&writes, 2, 8 * 1024, 512).unwrap();

        assert_eq!(summary.direct_writes, 4);
        assert_eq!(summary.largest_write, 32_768);
    }

    #[test]
    fn rejects_invalid_input() {
        assert_eq!(
            shape_summary(&[-1], 1, 8 * 1024, 512),
            Err(NbtGzipBufferShapeError::InvalidWriteLength)
        );
        assert_eq!(
            shape_summary(&[1], 0, 8 * 1024, 512),
            Err(NbtGzipBufferShapeError::InvalidRepeats)
        );
        assert_eq!(
            shape_summary(&[1], 1, 0, 512),
            Err(NbtGzipBufferShapeError::InvalidBufferSize)
        );
    }
}
