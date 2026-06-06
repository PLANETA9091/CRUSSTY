pub const SUMMARY_FIELDS: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceRulesSequenceArrayError {
    InvalidRuleCount,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SurfaceRulesSequenceArraySummary {
    pub count: u64,
    pub total: i64,
    pub checksum: u64,
    pub last_rule: i64,
}

#[derive(Clone, Copy)]
enum Mode {
    ListEnhanced,
    ListIndexed,
    ArrayForeach,
    ArrayIndexed,
}

#[inline]
pub fn list_enhanced_summary(
    iterations: usize,
    rules: usize,
) -> Result<SurfaceRulesSequenceArraySummary, SurfaceRulesSequenceArrayError> {
    sequence_summary(iterations, rules, Mode::ListEnhanced)
}

#[inline]
pub fn list_indexed_summary(
    iterations: usize,
    rules: usize,
) -> Result<SurfaceRulesSequenceArraySummary, SurfaceRulesSequenceArrayError> {
    sequence_summary(iterations, rules, Mode::ListIndexed)
}

#[inline]
pub fn array_foreach_summary(
    iterations: usize,
    rules: usize,
) -> Result<SurfaceRulesSequenceArraySummary, SurfaceRulesSequenceArrayError> {
    sequence_summary(iterations, rules, Mode::ArrayForeach)
}

#[inline]
pub fn array_indexed_summary(
    iterations: usize,
    rules: usize,
) -> Result<SurfaceRulesSequenceArraySummary, SurfaceRulesSequenceArrayError> {
    sequence_summary(iterations, rules, Mode::ArrayIndexed)
}

fn sequence_summary(
    iterations: usize,
    rules: usize,
    mode: Mode,
) -> Result<SurfaceRulesSequenceArraySummary, SurfaceRulesSequenceArrayError> {
    if rules == 0 {
        return Err(SurfaceRulesSequenceArrayError::InvalidRuleCount);
    }

    let mut total = 0i64;
    let mut checksum = 0u64;
    let mut last_rule = -1i64;

    for i in 0..iterations {
        let x = i as i32;
        let y = (i >> 4) as i32;
        let z = i.wrapping_mul(3) as i32;
        let selected = match mode {
            Mode::ListEnhanced => try_apply_foreach(rules, x, y, z),
            Mode::ListIndexed => try_apply_indexed(rules, x, y, z),
            Mode::ArrayForeach => try_apply_foreach(rules, x, y, z),
            Mode::ArrayIndexed => try_apply_indexed(rules, x, y, z),
        };

        let selected = selected as i64;
        total = total.wrapping_add(selected);
        last_rule = selected;
        checksum = mix64(
            checksum
                ^ (selected as u64)
                ^ ((i as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
                ^ ((rules as u64) << 17),
        );
    }

    Ok(SurfaceRulesSequenceArraySummary {
        count: iterations as u64,
        total,
        checksum,
        last_rule,
    })
}

#[inline]
fn try_apply_foreach(rules: usize, x: i32, y: i32, z: i32) -> usize {
    for index in 0..rules - 1 {
        if rule_matches(index, x, y, z) {
            return index;
        }
    }
    rules - 1
}

#[inline]
fn try_apply_indexed(rules: usize, x: i32, y: i32, z: i32) -> usize {
    let mut index = 0usize;
    while index + 1 < rules {
        if rule_matches(index, x, y, z) {
            return index;
        }
        index += 1;
    }
    rules - 1
}

#[inline]
fn rule_matches(index: usize, x: i32, y: i32, z: i32) -> bool {
    let value = x
        .wrapping_mul(31)
        .wrapping_add(y.wrapping_mul(17))
        .wrapping_add(z.wrapping_mul(13))
        .wrapping_add(index as i32);
    (value & 0x3fff) == 0
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
    fn sequence_shapes_match() {
        let list = list_enhanced_summary(100_000, 14).unwrap();
        assert_eq!(list, list_indexed_summary(100_000, 14).unwrap());
        assert_eq!(list, array_foreach_summary(100_000, 14).unwrap());
        assert_eq!(list, array_indexed_summary(100_000, 14).unwrap());
    }

    #[test]
    fn rejects_zero_rules() {
        assert_eq!(
            list_enhanced_summary(1, 0),
            Err(SurfaceRulesSequenceArrayError::InvalidRuleCount)
        );
    }
}
