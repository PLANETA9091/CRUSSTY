use std::error::Error;
use std::fmt;

pub const LIGHT_UPDATE_BYTES: usize = 2048;
const LIGHT_UPDATE_BYTES_VARINT: [u8; 2] = [0x80, 0x10];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    CountTooLarge,
    InvalidSectionShape,
    InvalidOffsetTable,
    InvalidUpdateBytes,
    DestinationTooSmall,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::CountTooLarge => f.write_str("count exceeds signed Minecraft VarInt range"),
            EncodeError::InvalidSectionShape => f.write_str("section encode input arrays have incompatible lengths"),
            EncodeError::InvalidOffsetTable => f.write_str("offset table is not monotonic or is out of range"),
            EncodeError::InvalidUpdateBytes => f.write_str("light update byte array length does not match update count"),
            EncodeError::DestinationTooSmall => f.write_str("destination buffer is smaller than the encoded packet"),
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

pub fn encode_section_data_into(input: &SectionEncodeInput<'_>, dst: &mut [u8]) -> EncodeResult<usize> {
    let len = encoded_section_data_len(input)?;
    if dst.len() < len {
        return Err(EncodeError::DestinationTooSmall);
    }

    let mut writer = SliceWriter::new(&mut dst[..len]);
    for section in 0..input.non_empty_counts.len() {
        write_i16_be_into(input.non_empty_counts[section], &mut writer)?;
        writer.push(input.state_bits[section])?;
        write_bytes_range_into(input.state_palette_offsets, input.state_palette_bytes, section, &mut writer)?;
        write_longs_range_into(input.state_storage_offsets, input.state_storage_longs, section, &mut writer)?;

        writer.push(input.biome_bits[section])?;
        write_bytes_range_into(input.biome_palette_offsets, input.biome_palette_bytes, section, &mut writer)?;
        write_longs_range_into(input.biome_storage_offsets, input.biome_storage_longs, section, &mut writer)?;
    }

    Ok(writer.written())
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

pub fn encode_light_data_into(input: &LightEncodeInput<'_>, dst: &mut [u8]) -> EncodeResult<usize> {
    let len = encoded_light_data_len(input)?;
    if dst.len() < len {
        return Err(EncodeError::DestinationTooSmall);
    }

    let mut writer = SliceWriter::new(&mut dst[..len]);
    write_long_array_into(input.sky_y_mask_longs, &mut writer)?;
    write_long_array_into(input.block_y_mask_longs, &mut writer)?;
    write_long_array_into(input.empty_sky_y_mask_longs, &mut writer)?;
    write_long_array_into(input.empty_block_y_mask_longs, &mut writer)?;
    write_light_updates_into(input.sky_updates, input.sky_update_count, &mut writer)?;
    write_light_updates_into(input.block_updates, input.block_update_count, &mut writer)?;

    Ok(writer.written())
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
    if value > i32::MAX as usize {
        return Err(EncodeError::CountTooLarge);
    }

    let mut remaining = value;
    let mut len = 1;
    while (remaining & !0x7F) != 0 {
        len += 1;
        remaining >>= 7;
    }
    Ok(len)
}

pub fn encoded_section_data_len(input: &SectionEncodeInput<'_>) -> EncodeResult<usize> {
    validate_section_input(input)?;

    let sections = input.non_empty_counts.len();
    let fixed = sections.checked_mul(4).ok_or(EncodeError::CountTooLarge)?;
    let state_storage = input
        .state_storage_longs
        .len()
        .checked_mul(8)
        .ok_or(EncodeError::CountTooLarge)?;
    let biome_storage = input
        .biome_storage_longs
        .len()
        .checked_mul(8)
        .ok_or(EncodeError::CountTooLarge)?;

    checked_sum(&[
        fixed,
        input.state_palette_bytes.len(),
        state_storage,
        input.biome_palette_bytes.len(),
        biome_storage,
    ])
}

pub fn encoded_light_data_len(input: &LightEncodeInput<'_>) -> EncodeResult<usize> {
    validate_update_bytes(input.sky_updates, input.sky_update_count)?;
    validate_update_bytes(input.block_updates, input.block_update_count)?;

    checked_sum(&[
        encoded_long_array_len(input.sky_y_mask_longs.len())?,
        encoded_long_array_len(input.block_y_mask_longs.len())?,
        encoded_long_array_len(input.empty_sky_y_mask_longs.len())?,
        encoded_long_array_len(input.empty_block_y_mask_longs.len())?,
        encoded_light_updates_len(input.sky_update_count)?,
        encoded_light_updates_len(input.block_update_count)?,
    ])
}

fn encoded_long_array_len(count: usize) -> EncodeResult<usize> {
    checked_sum(&[
        encoded_varint_len(count)?,
        count.checked_mul(8).ok_or(EncodeError::CountTooLarge)?,
    ])
}

fn encoded_light_updates_len(count: usize) -> EncodeResult<usize> {
    let layer_len = checked_sum(&[encoded_varint_len(LIGHT_UPDATE_BYTES)?, LIGHT_UPDATE_BYTES])?;
    checked_sum(&[
        encoded_varint_len(count)?,
        count.checked_mul(layer_len).ok_or(EncodeError::CountTooLarge)?,
    ])
}

fn checked_sum(values: &[usize]) -> EncodeResult<usize> {
    values.iter().try_fold(0usize, |sum, value| {
        sum.checked_add(*value).ok_or(EncodeError::CountTooLarge)
    })
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
        dst.extend_from_slice(&LIGHT_UPDATE_BYTES_VARINT);
        dst.extend_from_slice(layer);
    }
    Ok(())
}

struct SliceWriter<'a> {
    dst: &'a mut [u8],
    pos: usize,
}

impl<'a> SliceWriter<'a> {
    fn new(dst: &'a mut [u8]) -> Self {
        Self { dst, pos: 0 }
    }

    fn written(&self) -> usize {
        self.pos
    }

    fn push(&mut self, value: u8) -> EncodeResult<()> {
        if self.pos >= self.dst.len() {
            return Err(EncodeError::DestinationTooSmall);
        }

        self.dst[self.pos] = value;
        self.pos += 1;
        Ok(())
    }

    fn extend_from_slice(&mut self, values: &[u8]) -> EncodeResult<()> {
        let end = self.pos.checked_add(values.len()).ok_or(EncodeError::DestinationTooSmall)?;
        if end > self.dst.len() {
            return Err(EncodeError::DestinationTooSmall);
        }

        self.dst[self.pos..end].copy_from_slice(values);
        self.pos = end;
        Ok(())
    }
}

fn write_varint_into(value: usize, dst: &mut SliceWriter<'_>) -> EncodeResult<()> {
    if value > i32::MAX as usize {
        return Err(EncodeError::CountTooLarge);
    }

    let mut remaining = value as u32;
    loop {
        if (remaining & !0x7F) == 0 {
            dst.push(remaining as u8)?;
            return Ok(());
        }

        dst.push(((remaining & 0x7F) | 0x80) as u8)?;
        remaining >>= 7;
    }
}

fn write_i16_be_into(value: i16, dst: &mut SliceWriter<'_>) -> EncodeResult<()> {
    dst.extend_from_slice(&value.to_be_bytes())
}

fn write_i64_be_into(value: i64, dst: &mut SliceWriter<'_>) -> EncodeResult<()> {
    dst.extend_from_slice(&value.to_be_bytes())
}

fn write_long_array_into(values: &[i64], dst: &mut SliceWriter<'_>) -> EncodeResult<()> {
    write_varint_into(values.len(), dst)?;
    for &value in values {
        write_i64_be_into(value, dst)?;
    }
    Ok(())
}

fn write_light_updates_into(bytes: &[u8], count: usize, dst: &mut SliceWriter<'_>) -> EncodeResult<()> {
    write_varint_into(count, dst)?;
    for layer in bytes.chunks_exact(LIGHT_UPDATE_BYTES) {
        dst.extend_from_slice(&LIGHT_UPDATE_BYTES_VARINT)?;
        dst.extend_from_slice(layer)?;
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

fn write_bytes_range_into(
    offsets: &[i32],
    bytes: &[u8],
    section: usize,
    dst: &mut SliceWriter<'_>,
) -> EncodeResult<()> {
    let (start, end) = offset_range(offsets, section)?;
    dst.extend_from_slice(&bytes[start..end])
}

fn write_longs_range_into(
    offsets: &[i32],
    longs: &[i64],
    section: usize,
    dst: &mut SliceWriter<'_>,
) -> EncodeResult<()> {
    let (start, end) = offset_range(offsets, section)?;
    for &value in &longs[start..end] {
        write_i64_be_into(value, dst)?;
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
        assert_eq!(encoded_varint_len(0).unwrap(), 1);
        assert_eq!(encoded_varint_len(127).unwrap(), 1);
        assert_eq!(encoded_varint_len(128).unwrap(), 2);
        assert_eq!(encoded_varint_len(2_097_152).unwrap(), 4);
        assert_eq!(encoded_varint_len(i32::MAX as usize).unwrap(), 5);
        assert_eq!(encoded_varint_len(i32::MAX as usize + 1), Err(EncodeError::CountTooLarge));

        out.clear();
        write_varint(LIGHT_UPDATE_BYTES, &mut out).unwrap();
        assert_eq!(out, LIGHT_UPDATE_BYTES_VARINT);
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
        assert_eq!(encoded_light_data_len(&input).unwrap(), out.len());
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
    fn encodes_light_update_packet_data_into_existing_buffer() {
        let sky = patterned_light_bytes(0x13, 3);
        let block = patterned_light_bytes(0x91, 2);
        let input = LightEncodeInput {
            sky_y_mask_longs: &[0b111, 0],
            block_y_mask_longs: &[0b11],
            empty_sky_y_mask_longs: &[0],
            empty_block_y_mask_longs: &[],
            sky_updates: &sky,
            sky_update_count: 3,
            block_updates: &block,
            block_update_count: 2,
        };

        let mut expected = Vec::new();
        encode_light_data(&input, &mut expected).unwrap();

        let mut actual = vec![0xCC; expected.len() + 8];
        let written = encode_light_data_into(&input, &mut actual).unwrap();

        assert_eq!(written, expected.len());
        assert_eq!(&actual[..written], &expected);
        assert_eq!(&actual[written..], &[0xCC; 8]);
    }

    #[test]
    fn encode_light_update_into_rejects_small_destination() {
        let sky = vec![0x5Au8; LIGHT_UPDATE_BYTES];
        let input = LightEncodeInput {
            sky_y_mask_longs: &[1],
            block_y_mask_longs: &[],
            empty_sky_y_mask_longs: &[],
            empty_block_y_mask_longs: &[],
            sky_updates: &sky,
            sky_update_count: 1,
            block_updates: &[],
            block_update_count: 0,
        };
        let len = encoded_light_data_len(&input).unwrap();
        let mut actual = vec![0u8; len - 1];

        assert_eq!(encode_light_data_into(&input, &mut actual), Err(EncodeError::DestinationTooSmall));
    }

    #[test]
    fn encodes_section_data_into_existing_buffer() {
        let input = section_fixture_input();

        let mut expected = Vec::new();
        encode_section_data(&input, &mut expected).unwrap();

        let mut actual = vec![0xCC; expected.len() + 8];
        let written = encode_section_data_into(&input, &mut actual).unwrap();

        assert_eq!(written, expected.len());
        assert_eq!(&actual[..written], &expected);
        assert_eq!(&actual[written..], &[0xCC; 8]);
    }

    #[test]
    fn encode_section_data_into_rejects_small_destination() {
        let input = section_fixture_input();
        let len = encoded_section_data_len(&input).unwrap();
        let mut actual = vec![0u8; len - 1];

        assert_eq!(encode_section_data_into(&input, &mut actual), Err(EncodeError::DestinationTooSmall));
    }

    #[test]
    fn encodes_light_update_parity_for_empty_and_max_masks() {
        let empty = LightEncodeInput {
            sky_y_mask_longs: &[],
            block_y_mask_longs: &[],
            empty_sky_y_mask_longs: &[],
            empty_block_y_mask_longs: &[],
            sky_updates: &[],
            sky_update_count: 0,
            block_updates: &[],
            block_update_count: 0,
        };
        assert_light_matches_reference(&empty);

        let mut out = Vec::new();
        encode_light_data(&empty, &mut out).unwrap();
        assert_eq!(encoded_light_data_len(&empty).unwrap(), out.len());
        assert_eq!(out, [0, 0, 0, 0, 0, 0]);

        let sky = patterned_light_bytes(0x21, 2);
        let block = patterned_light_bytes(0xA7, 1);
        let max_masks = LightEncodeInput {
            sky_y_mask_longs: &[i64::MAX, -1],
            block_y_mask_longs: &[-1],
            empty_sky_y_mask_longs: &[i64::MIN, i64::MAX],
            empty_block_y_mask_longs: &[0x0123_4567_89AB_CDEF],
            sky_updates: &sky,
            sky_update_count: 2,
            block_updates: &block,
            block_update_count: 1,
        };
        assert_light_matches_reference(&max_masks);
    }

    fn assert_light_matches_reference(input: &LightEncodeInput<'_>) {
        let expected = reference_encode_light(input);
        let mut actual = Vec::new();
        let written = encode_light_data(input, &mut actual).unwrap();

        assert_eq!(written, actual.len());
        assert_eq!(actual, expected);
    }

    fn reference_encode_light(input: &LightEncodeInput<'_>) -> Vec<u8> {
        let mut out = Vec::new();
        ref_write_long_array(input.sky_y_mask_longs, &mut out);
        ref_write_long_array(input.block_y_mask_longs, &mut out);
        ref_write_long_array(input.empty_sky_y_mask_longs, &mut out);
        ref_write_long_array(input.empty_block_y_mask_longs, &mut out);
        ref_write_light_updates(input.sky_updates, input.sky_update_count, &mut out);
        ref_write_light_updates(input.block_updates, input.block_update_count, &mut out);
        out
    }

    fn ref_write_long_array(values: &[i64], out: &mut Vec<u8>) {
        ref_write_varint(values.len() as i32, out);
        for value in values {
            let bits = *value as u64;
            for shift in (0..=56).rev().step_by(8) {
                out.push((bits >> shift) as u8);
            }
        }
    }

    fn ref_write_light_updates(bytes: &[u8], count: usize, out: &mut Vec<u8>) {
        ref_write_varint(count as i32, out);
        for i in 0..count {
            ref_write_varint(LIGHT_UPDATE_BYTES as i32, out);
            let start = i * LIGHT_UPDATE_BYTES;
            out.extend_from_slice(&bytes[start..start + LIGHT_UPDATE_BYTES]);
        }
    }

    fn ref_write_varint(mut value: i32, out: &mut Vec<u8>) {
        while (value & -128) != 0 {
            out.push(((value & 127) | 128) as u8);
            value = ((value as u32) >> 7) as i32;
        }
        out.push(value as u8);
    }

    fn patterned_light_bytes(seed: u8, count: usize) -> Vec<u8> {
        (0..count * LIGHT_UPDATE_BYTES)
            .map(|index| seed.wrapping_add((index as u8).wrapping_mul(31)))
            .collect()
    }

    fn section_fixture_input() -> SectionEncodeInput<'static> {
        SectionEncodeInput {
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
        }
    }

    #[test]
    fn encodes_section_data_from_preencoded_palette_parts() {
        let input = section_fixture_input();

        let mut out = Vec::new();
        encode_section_data(&input, &mut out).unwrap();
        assert_eq!(encoded_section_data_len(&input).unwrap(), out.len());

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
