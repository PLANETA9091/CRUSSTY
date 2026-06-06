const CHUNK_MASK: i64 = 0xFFFF_FFFF;
const SECTION_XZ_MASK: i64 = 0x3F_FFFF;
const SECTION_Y_MASK: i64 = 0xF_FFFF;

#[inline]
pub fn chunk_as_long(x: i32, z: i32) -> i64 {
    ((x as i64) & CHUNK_MASK) | (((z as i64) & CHUNK_MASK) << 32)
}

#[inline]
pub fn chunk_x(packed: i64) -> i32 {
    (packed & CHUNK_MASK) as i32
}

#[inline]
pub fn chunk_z(packed: i64) -> i32 {
    ((packed >> 32) & CHUNK_MASK) as i32
}

#[inline]
pub fn chunk_hash(x: i32, z: i32) -> i32 {
    let x_hash = 1_664_525i32.wrapping_mul(x).wrapping_add(1_013_904_223);
    let z_hash = 1_664_525i32
        .wrapping_mul(z ^ -559_038_737i32)
        .wrapping_add(1_013_904_223);
    x_hash ^ z_hash
}

#[inline]
pub fn section_as_long(x: i32, y: i32, z: i32) -> i64 {
    (((x as i64) & SECTION_XZ_MASK) << 42)
        | ((y as i64) & SECTION_Y_MASK)
        | (((z as i64) & SECTION_XZ_MASK) << 20)
}

#[inline]
pub fn block_pos_as_section_long(x: i32, y: i32, z: i32) -> i64 {
    section_as_long(x >> 4, y >> 4, z >> 4)
}

#[inline]
pub fn section_x(packed: i64) -> i32 {
    (packed << 0 >> 42) as i32
}

#[inline]
pub fn section_y(packed: i64) -> i32 {
    (packed << 44 >> 44) as i32
}

#[inline]
pub fn section_z(packed: i64) -> i32 {
    (packed << 22 >> 42) as i32
}

#[inline]
pub fn section_to_chunk(packed: i64) -> i64 {
    chunk_as_long(section_x(packed), section_z(packed))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHUNK_CASES: &[(i32, i32)] = &[
        (0, 0),
        (1, 2),
        (-1, -2),
        (1_875_066, 1_875_066),
        (i32::MAX, i32::MIN),
        (i32::MIN, i32::MAX),
    ];

    const SECTION_CASES: &[(i32, i32, i32)] = &[
        (0, 0, 0),
        (1, 2, 3),
        (-1, -2, -3),
        (2_097_151, 524_287, -2_097_152),
        (-2_097_152, -524_288, 2_097_151),
    ];

    #[test]
    fn chunk_pack_round_trips_all_i32_bits() {
        for &(x, z) in CHUNK_CASES {
            let packed = chunk_as_long(x, z);
            assert_eq!(chunk_x(packed), x);
            assert_eq!(chunk_z(packed), z);
        }
    }

    #[test]
    fn chunk_hash_matches_java_overflow_cases() {
        assert_eq!(chunk_hash(0, 0), 1_455_762_653);
        assert_eq!(chunk_hash(1, 2), 1_458_653_700);
        assert_eq!(chunk_hash(-1, -2), 845_646_446);
        assert_eq!(chunk_hash(i32::MAX, i32::MIN), 1_459_507_920);
    }

    #[test]
    fn section_pack_round_trips_signed_field_range() {
        for &(x, y, z) in SECTION_CASES {
            let packed = section_as_long(x, y, z);
            assert_eq!(section_x(packed), x);
            assert_eq!(section_y(packed), y);
            assert_eq!(section_z(packed), z);
            assert_eq!(section_to_chunk(packed), chunk_as_long(x, z));
        }
    }

    #[test]
    fn block_position_to_section_long_uses_arithmetic_shift() {
        let packed = block_pos_as_section_long(-1, -16, 31);
        assert_eq!(section_x(packed), -1);
        assert_eq!(section_y(packed), -1);
        assert_eq!(section_z(packed), 1);
    }
}
