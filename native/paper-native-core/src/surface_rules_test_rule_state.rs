pub const SUMMARY_FIELDS: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceRulesTestRuleStateError {
    InvalidPeriod,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SurfaceRulesTestRuleStateSummary {
    pub count: u64,
    pub hits: u64,
    pub checksum: u64,
    pub last_hit: u64,
}

#[inline]
pub fn old_state_rule_summary(
    iterations: usize,
    period: usize,
) -> Result<SurfaceRulesTestRuleStateSummary, SurfaceRulesTestRuleStateError> {
    state_rule_summary_modulo(iterations, period)
}

#[inline]
pub fn new_state_rule_summary(
    iterations: usize,
    period: usize,
) -> Result<SurfaceRulesTestRuleStateSummary, SurfaceRulesTestRuleStateError> {
    state_rule_summary_countdown(iterations, period)
}

fn state_rule_summary_modulo(
    iterations: usize,
    period: usize,
) -> Result<SurfaceRulesTestRuleStateSummary, SurfaceRulesTestRuleStateError> {
    if period == 0 {
        return Err(SurfaceRulesTestRuleStateError::InvalidPeriod);
    }

    let mut counter = 0usize;
    let mut hits = 0u64;
    let mut checksum = 0u64;
    let mut last_hit = 0u64;

    for i in 0..iterations {
        counter += 1;
        let hit = (counter % period) != 0;
        if hit {
            hits = hits.wrapping_add(1);
        }
        last_hit = u64::from(hit);
        checksum = mix64(
            checksum
                ^ last_hit
                ^ ((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((period as u64) << 23),
        );
    }

    Ok(SurfaceRulesTestRuleStateSummary {
        count: iterations as u64,
        hits,
        checksum,
        last_hit,
    })
}

fn state_rule_summary_countdown(
    iterations: usize,
    period: usize,
) -> Result<SurfaceRulesTestRuleStateSummary, SurfaceRulesTestRuleStateError> {
    if period == 0 {
        return Err(SurfaceRulesTestRuleStateError::InvalidPeriod);
    }

    let mut remaining = period;
    let mut hits = 0u64;
    let mut checksum = 0u64;
    let mut last_hit = 0u64;

    for i in 0..iterations {
        remaining -= 1;
        let hit = remaining != 0;
        if !hit {
            remaining = period;
        } else {
            hits = hits.wrapping_add(1);
        }
        last_hit = u64::from(hit);
        checksum = mix64(
            checksum
                ^ last_hit
                ^ ((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                ^ ((period as u64) << 23),
        );
    }

    Ok(SurfaceRulesTestRuleStateSummary {
        count: iterations as u64,
        hits,
        checksum,
        last_hit,
    })
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
    fn old_and_new_match() {
        for period in [1, 2, 3, 7, 97] {
            assert_eq!(
                old_state_rule_summary(100_000, period).unwrap(),
                new_state_rule_summary(100_000, period).unwrap()
            );
        }
    }

    #[test]
    fn hit_counts_match_periodic_condition() {
        assert_eq!(old_state_rule_summary(10, 2).unwrap().hits, 5);
        assert_eq!(old_state_rule_summary(10, 7).unwrap().hits, 9);
    }

    #[test]
    fn rejects_zero_period() {
        assert_eq!(
            old_state_rule_summary(1, 0),
            Err(SurfaceRulesTestRuleStateError::InvalidPeriod)
        );
    }
}
