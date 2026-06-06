pub const SUMMARY_FIELDS: usize = 5;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const COPIED_TAG: u64 = 0x4445_464C_434F_5059;
const SLICE_TAG: u64 = 0x4445_464C_534C_4943;
const PAYLOAD_TAG: u64 = 0x4445_464C_5041_594C;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DeflaterInputShapeSummary {
    pub visits: u64,
    pub total_bytes: u64,
    pub copied_bytes: u64,
    pub payload_checksum: u64,
    pub shape_checksum: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeflaterInputShapeError {
    InvalidInputLength,
    InvalidIterations,
    InvalidSlice,
}

pub fn copied_summary(
    payload: &[u8],
    offsets: &[i32],
    lengths: &[i32],
    iterations: usize,
) -> Result<DeflaterInputShapeSummary, DeflaterInputShapeError> {
    run_summary(payload, offsets, lengths, iterations, true)
}

pub fn slice_summary(
    payload: &[u8],
    offsets: &[i32],
    lengths: &[i32],
    iterations: usize,
) -> Result<DeflaterInputShapeSummary, DeflaterInputShapeError> {
    run_summary(payload, offsets, lengths, iterations, false)
}

fn run_summary(
    payload: &[u8],
    offsets: &[i32],
    lengths: &[i32],
    iterations: usize,
    copied: bool,
) -> Result<DeflaterInputShapeSummary, DeflaterInputShapeError> {
    if offsets.len() != lengths.len() {
        return Err(DeflaterInputShapeError::InvalidInputLength);
    }
    if iterations == 0 {
        return Err(DeflaterInputShapeError::InvalidIterations);
    }

    let tag = if copied { COPIED_TAG } else { SLICE_TAG };
    let mut summary = DeflaterInputShapeSummary {
        payload_checksum: mix64(PAYLOAD_TAG ^ offsets.len() as u64 ^ ((iterations as u64) << 32)),
        shape_checksum: mix64(tag ^ payload.len() as u64),
        ..DeflaterInputShapeSummary::default()
    };

    for iteration in 0..iterations {
        for (index, (&offset, &length)) in offsets.iter().zip(lengths.iter()).enumerate() {
            let segment = checked_segment(payload, offset, length)?;
            let digest = if copied {
                let owned = segment.to_vec();
                digest_bytes(&owned)
            } else {
                digest_bytes(segment)
            };

            let length = segment.len() as u64;
            summary.visits = summary.visits.wrapping_add(1);
            summary.total_bytes = summary.total_bytes.wrapping_add(length);
            if copied {
                summary.copied_bytes = summary.copied_bytes.wrapping_add(length);
            }
            summary.payload_checksum = mix64(
                summary.payload_checksum
                    ^ digest
                    ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                    ^ ((index as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)),
            );
            summary.shape_checksum = mix64(
                summary.shape_checksum
                    ^ tag
                    ^ ((offset as u32 as u64) << 1)
                    ^ ((length as u64) << 17)
                    ^ summary.copied_bytes.rotate_left(13),
            );
        }
    }

    Ok(summary)
}

fn checked_segment<'a>(
    payload: &'a [u8],
    offset: i32,
    length: i32,
) -> Result<&'a [u8], DeflaterInputShapeError> {
    let offset = usize::try_from(offset).map_err(|_| DeflaterInputShapeError::InvalidSlice)?;
    let length = usize::try_from(length).map_err(|_| DeflaterInputShapeError::InvalidSlice)?;
    let end = offset
        .checked_add(length)
        .ok_or(DeflaterInputShapeError::InvalidSlice)?;
    payload
        .get(offset..end)
        .ok_or(DeflaterInputShapeError::InvalidSlice)
}

#[inline]
fn digest_bytes(input: &[u8]) -> u64 {
    let mut digest = mix64(input.len() as u64 ^ 0xD1B5_4A32_D192_ED03);
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
    fn copied_and_slice_payloads_match_but_shape_differs() {
        let payload = b"aaaabbbbccccddddeeee";
        let offsets = [0, 4, 8, 12];
        let lengths = [4, 4, 4, 4];
        let copied = copied_summary(payload, &offsets, &lengths, 3).unwrap();
        let sliced = slice_summary(payload, &offsets, &lengths, 3).unwrap();

        assert_eq!(copied.visits, sliced.visits);
        assert_eq!(copied.total_bytes, sliced.total_bytes);
        assert_eq!(copied.payload_checksum, sliced.payload_checksum);
        assert_eq!(copied.copied_bytes, copied.total_bytes);
        assert_eq!(sliced.copied_bytes, 0);
        assert_ne!(copied.shape_checksum, sliced.shape_checksum);
    }

    #[test]
    fn rejects_invalid_slice() {
        assert_eq!(
            copied_summary(b"abc", &[1], &[8], 1),
            Err(DeflaterInputShapeError::InvalidSlice)
        );
    }

    #[test]
    fn rejects_zero_iterations() {
        assert_eq!(
            slice_summary(b"abc", &[0], &[3], 0),
            Err(DeflaterInputShapeError::InvalidIterations)
        );
    }
}
