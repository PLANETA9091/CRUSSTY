use std::error::Error;
use std::fmt;

pub const LIGHT_UPDATE_BYTES: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    CountTooLarge,
    InvalidSectionShape,
    InvalidOffsetTable,
    InvalidUpdateBytes,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::CountTooLarge => f.write_str("count exceeds signed Minecraft VarInt range"),
            EncodeError::InvalidSectionShape => f.write_str("section encode input arrays have incompatible lengths"),
            EncodeError::InvalidOffsetTable => f.write_str("offset table is not monotonic or is out of range"),
            EncodeError::InvalidUpdateBytes => f.write_str("light update byte array length does not match update count"),
        }
    }
}

impl Error for EncodeError {}

pub type EncodeResult<T> = Result<T, EncodeError>;

pub struct SectionEncodeInput<'a> {
    pub non_empty_counts: &'a [i16],
    pub state_bits: &'a [u8],
    pub state_palette_offsets: &'a [i32],
    pub state_palette_bytes: &'a [u8],
    pub state_storage_offsets: &'a [i32],
    pub state_storage_longs: &'a [i64],
    pub biome_bits: &'a [u8],
    pub biome_palette_offsets: &'a [i32],
    pub biome_palette_bytes: &'a [u8],
    pub biome_storage_offsets: &'a [i32],
    pub biome_storage_longs: &'a [i64],
}

pub struct LightEncodeInput<'a> {
    pub sky_y_mask_longs: &'a [i64],
    pub block_y_mask_longs: &'a [i64],
    pub empty_sky_y_mask_longs: &'a [i64],
    pub empty_block_y_mask_longs: &'a [i64],
    pub sky_updates: &'a [u8],
    pub sky_update_count: usize,
    pub block_updates: &'a [u8],
    pub block_update_count: usize,
}

pub fn encode_section_data(input: &SectionEncodeInput<'_>, dst: &mut Vec<u8>) -> EncodeResult<usize> {
    let start = dst.len();
    validate_section_input(input)?;

    for section in 0..input.non_empty_counts.len() {
        write_i16_be(input.non_empty_counts[section], dst);
        dst.push(input.state_bits[section]);
        write_bytes_range(input.state_palette_offsets, input.state_palette_bytes, section, dst)?;
        write_longs_range(input.state_storage_offsets, input.state_storage_longs, section, dst)?;

        dst.push(input.biome_bits[section]);
        write_bytes_range(input.biome_palette_offsets, input.biome_palette_bytes, section, dst)?;
        write_longs_range(input.biome_storage_offsets, input.biome_storage_longs, section, dst)?;
    }

    Ok(dst.len() - start)
}

pub fn encode_light_data(input: &LightEncodeInput<'_>, dst: &mut Vec<u8>) -> EncodeResult<usize> {
    let start = dst.len();

    validate_update_bytes(input.sky_updates, input.sky_update_count)?;
    validate_update_bytes(input.block_updates, input.block_update_count)?;

    write_long_array(input.sky_y_mask_longs, dst)?;
    write_long_array(input.block_y_mask_longs, dst)?;
    write_long_array(input.empty_sky_y_mask_longs, dst)?;
    write_long_array(input.empty_block_y_mask_longs, dst)?;
    write_light_updates(input.sky_updates, input.sky_update_count, dst)?;
    write_light_updates(input.block_updates, input.block_update_count, dst)?;

    Ok(dst.len() - start)
}

pub fn encode_trimmed_bitset_words(words: &[i64], dst: &mut Vec<u8>) -> EncodeResult<usize> {
    let trimmed_len = words
        .iter()
        .rposition(|word| *word != 0)
        .map_or(0, |index| index + 1);
    let start = dst.len();
    write_long_array(&words[..trimmed_len], dst)?;
    Ok(dst.len() - start)
}

pub fn write_varint(value: usize, dst: &mut Vec<u8>) -> EncodeResult<usize> {
    if value > i32::MAX as usize {
        return Err(EncodeError::CountTooLarge);
    }

    let start = dst.len();
    let mut remaining = value as u32;
    loop {
        if (remaining & !0x7F) == 0 {
            dst.push(remaining as u8);
            break;
        }

        dst.push(((remaining & 0x7F) | 0x80) as u8);
        remaining >>= 7;
    }

    Ok(dst.len() - start)
}

pub fn encoded_varint_len(value: usize) -> EncodeResult<usize> {
    let mut tmp = Vec::with_capacity(5);
    write_varint(value, &mut tmp)
}

fn validate_section_input(input: &SectionEncodeInput<'_>) -> EncodeResult<()> {
    let sections = input.non_empty_counts.len();
    if input.state_bits.len() != sections || input.biome_bits.len() != sections {
        return Err(EncodeError::InvalidSectionShape);
    }

    validate_offsets(input.state_palette_offsets, input.state_palette_bytes.len(), sections)?;
    validate_offsets(input.state_storage_offsets, input.state_storage_longs.len(), sections)?;
    validate_offsets(input.biome_palette_offsets, input.biome_palette_bytes.len(), sections)?;
    validate_offsets(input.biome_storage_offsets, input.biome_storage_longs.len(), sections)?;

    Ok(())
}

fn validate_offsets(offsets: &[i32], len: usize, sections: usize) -> EncodeResult<()> {
    if offsets.len() != sections + 1 {
        return Err(EncodeError::InvalidOffsetTable);
    }

    let mut previous = 0usize;
    for &offset in offsets {
        let current = usize::try_from(offset).map_err(|_| EncodeError::InvalidOffsetTable)?;
        if current < previous || current > len {
            return Err(EncodeError::InvalidOffsetTable);
        }
        previous = current;
    }

    if previous != len {
        return Err(EncodeError::InvalidOffsetTable);
    }

    Ok(())
}

fn validate_update_bytes(bytes: &[u8], count: usize) -> EncodeResult<()> {
    match count.checked_mul(LIGHT_UPDATE_BYTES) {
        Some(expected) if expected == bytes.len() => Ok(()),
        _ => Err(EncodeError::InvalidUpdateBytes),
    }
}

fn write_i16_be(value: i16, dst: &mut Vec<u8>) {
    dst.extend_from_slice(&value.to_be_bytes());
}

fn write_i64_be(value: i64, dst: &mut Vec<u8>) {
    dst.extend_from_slice(&value.to_be_bytes());
}

fn write_long_array(values: &[i64], dst: &mut Vec<u8>) -> EncodeResult<()> {
    write_varint(values.len(), dst)?;
    for &value in values {
        write_i64_be(value, dst);
    }
    Ok(())
}

fn write_light_updates(bytes: &[u8], count: usize, dst: &mut Vec<u8>) -> EncodeResult<()> {
    write_varint(count, dst)?;
    for layer in bytes.chunks_exact(LIGHT_UPDATE_BYTES) {
        write_varint(LIGHT_UPDATE_BYTES, dst)?;
        dst.extend_from_slice(layer);
    }
    Ok(())
}

fn write_bytes_range(offsets: &[i32], bytes: &[u8], section: usize, dst: &mut Vec<u8>) -> EncodeResult<()> {
    let (start, end) = offset_range(offsets, section)?;
    dst.extend_from_slice(&bytes[start..end]);
    Ok(())
}

fn write_longs_range(offsets: &[i32], longs: &[i64], section: usize, dst: &mut Vec<u8>) -> EncodeResult<()> {
    let (start, end) = offset_range(offsets, section)?;
    for &value in &longs[start..end] {
        write_i64_be(value, dst);
    }
    Ok(())
}

fn offset_range(offsets: &[i32], section: usize) -> EncodeResult<(usize, usize)> {
    let start = usize::try_from(offsets[section]).map_err(|_| EncodeError::InvalidOffsetTable)?;
    let end = usize::try_from(offsets[section + 1]).map_err(|_| EncodeError::InvalidOffsetTable)?;
    if start > end {
        return Err(EncodeError::InvalidOffsetTable);
    }
    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_matches_minecraft_lengths() {
        let mut out = Vec::new();
        write_varint(0, &mut out).unwrap();
        write_varint(127, &mut out).unwrap();
        write_varint(128, &mut out).unwrap();
        write_varint(2_097_152, &mut out).unwrap();
        assert_eq!(out, [0x00, 0x7F, 0x80, 0x01, 0x80, 0x80, 0x80, 0x01]);
    }

    #[test]
    fn trims_bitset_words_like_bitset_to_long_array() {
        let mut out = Vec::new();
        encode_trimmed_bitset_words(&[0x10, 0, 0], &mut out).unwrap();
        assert_eq!(out, [1, 0, 0, 0, 0, 0, 0, 0, 0x10]);

        out.clear();
        encode_trimmed_bitset_words(&[0, 0], &mut out).unwrap();
        assert_eq!(out, [0]);
    }

    #[test]
    fn encodes_light_update_packet_data_shape() {
        let sky = vec![0x5Au8; LIGHT_UPDATE_BYTES * 2];
        let block = vec![0xA5u8; LIGHT_UPDATE_BYTES];
        let input = LightEncodeInput {
            sky_y_mask_longs: &[0b101],
            block_y_mask_longs: &[0b10],
            empty_sky_y_mask_longs: &[],
            empty_block_y_mask_longs: &[0b1000],
            sky_updates: &sky,
            sky_update_count: 2,
            block_updates: &block,
            block_update_count: 1,
        };

        let mut out = Vec::new();
        let written = encode_light_data(&input, &mut out).unwrap();
        assert_eq!(written, out.len());
        assert_eq!(out[0], 1);
        assert_eq!(&out[1..9], &5i64.to_be_bytes());
        assert_eq!(out[9], 1);
        assert_eq!(&out[10..18], &2i64.to_be_bytes());
        assert_eq!(out[18], 0);
        assert_eq!(out[19], 1);
        assert_eq!(&out[20..28], &8i64.to_be_bytes());
        assert_eq!(out[28], 2);
        assert_eq!(&out[29..31], &[0x80, 0x10]);
        assert_eq!(out[31], 0x5A);
        let second_layer_prefix = 31 + LIGHT_UPDATE_BYTES;
        assert_eq!(&out[second_layer_prefix..second_layer_prefix + 2], &[0x80, 0x10]);
        let block_count_index = second_layer_prefix + 2 + LIGHT_UPDATE_BYTES;
        assert_eq!(out[block_count_index], 1);
        assert_eq!(&out[block_count_index + 1..block_count_index + 3], &[0x80, 0x10]);
    }

    #[test]
    fn encodes_section_data_from_preencoded_palette_parts() {
        let input = SectionEncodeInput {
            non_empty_counts: &[7, 9],
            state_bits: &[4, 5],
            state_palette_offsets: &[0, 3, 7],
            state_palette_bytes: &[0x02, 0x00, 0x01, 0x03, 0x00, 0x01, 0x02],
            state_storage_offsets: &[0, 2, 3],
            state_storage_longs: &[0x1122, -1, 0x3344],
            biome_bits: &[1, 2],
            biome_palette_offsets: &[0, 2, 5],
            biome_palette_bytes: &[0x01, 0x2A, 0x02, 0x2B, 0x2C],
            biome_storage_offsets: &[0, 0, 1],
            biome_storage_longs: &[0x5566],
        };

        let mut out = Vec::new();
        encode_section_data(&input, &mut out).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&7i16.to_be_bytes());
        expected.push(4);
        expected.extend_from_slice(&[0x02, 0x00, 0x01]);
        expected.extend_from_slice(&0x1122i64.to_be_bytes());
        expected.extend_from_slice(&(-1i64).to_be_bytes());
        expected.push(1);
        expected.extend_from_slice(&[0x01, 0x2A]);
        expected.extend_from_slice(&9i16.to_be_bytes());
        expected.push(5);
        expected.extend_from_slice(&[0x03, 0x00, 0x01, 0x02]);
        expected.extend_from_slice(&0x3344i64.to_be_bytes());
        expected.push(2);
        expected.extend_from_slice(&[0x02, 0x2B, 0x2C]);
        expected.extend_from_slice(&0x5566i64.to_be_bytes());

        assert_eq!(out, expected);
    }
}
