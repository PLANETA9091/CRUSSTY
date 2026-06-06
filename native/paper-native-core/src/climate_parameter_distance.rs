use crate::climate::{ClimateError, PARAMETER_COUNT};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistanceKind {
    Old,
    Branch,
    SubtractFirst,
}

#[inline]
pub fn old_distance_sum(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
) -> Result<i64, ClimateError> {
    distance_sum(node_mins, node_maxs, queries, DistanceKind::Old)
}

#[inline]
pub fn branch_distance_sum(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
) -> Result<i64, ClimateError> {
    distance_sum(node_mins, node_maxs, queries, DistanceKind::Branch)
}

#[inline]
pub fn subtract_first_distance_sum(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
) -> Result<i64, ClimateError> {
    distance_sum(node_mins, node_maxs, queries, DistanceKind::SubtractFirst)
}

pub fn distance_sum(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
    kind: DistanceKind,
) -> Result<i64, ClimateError> {
    if node_mins.len() != node_maxs.len()
        || node_mins.len() % PARAMETER_COUNT != 0
        || queries.len() % PARAMETER_COUNT != 0
    {
        return Err(ClimateError::InvalidInputLength);
    }

    let mut sum = 0i64;
    for query in queries.chunks_exact(PARAMETER_COUNT) {
        for (node_min, node_max) in node_mins
            .chunks_exact(PARAMETER_COUNT)
            .zip(node_maxs.chunks_exact(PARAMETER_COUNT))
        {
            sum = sum.wrapping_add(node_score(node_min, node_max, query, kind));
        }
    }
    Ok(sum)
}

#[inline]
fn node_score(node_mins: &[i64], node_maxs: &[i64], query: &[i64], kind: DistanceKind) -> i64 {
    let mut node_sum = 0i64;
    for parameter in 0..PARAMETER_COUNT {
        let distance = match kind {
            DistanceKind::Old => {
                old_distance(node_mins[parameter], node_maxs[parameter], query[parameter])
            }
            DistanceKind::Branch => {
                branch_distance(node_mins[parameter], node_maxs[parameter], query[parameter])
            }
            DistanceKind::SubtractFirst => subtract_first_distance(
                node_mins[parameter],
                node_maxs[parameter],
                query[parameter],
            ),
        };
        node_sum = node_sum.wrapping_add(distance.wrapping_mul(distance));
    }
    node_sum
}

#[inline]
pub fn old_distance(min: i64, max: i64, value: i64) -> i64 {
    let above = value.wrapping_sub(max);
    let below = min.wrapping_sub(value);
    if above > 0 {
        above
    } else if below > 0 {
        below
    } else {
        0
    }
}

#[inline]
pub fn branch_distance(min: i64, max: i64, value: i64) -> i64 {
    if value > max {
        value.wrapping_sub(max)
    } else if value < min {
        min.wrapping_sub(value)
    } else {
        0
    }
}

#[inline]
pub fn subtract_first_distance(min: i64, max: i64, value: i64) -> i64 {
    let above = value.wrapping_sub(max);
    if above > 0 {
        return above;
    }

    let below = min.wrapping_sub(value);
    if below > 0 {
        below
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_variants_match_reference_cases() {
        for &(min, max, value, expected) in &[
            (10, 20, 5, 5),
            (10, 20, 10, 0),
            (10, 20, 15, 0),
            (10, 20, 20, 0),
            (10, 20, 25, 5),
        ] {
            assert_eq!(old_distance(min, max, value), expected);
            assert_eq!(branch_distance(min, max, value), expected);
            assert_eq!(subtract_first_distance(min, max, value), expected);
        }
    }

    #[test]
    fn distance_sums_match_across_variants() {
        let node_mins = [0, 10, 20, 30, 40, 50, 60];
        let node_maxs = [5, 15, 25, 35, 45, 55, 65];
        let queries = [3, 12, 22, 33, 43, 52, 70];

        assert_eq!(old_distance_sum(&node_mins, &node_maxs, &queries).unwrap(), 25);
        assert_eq!(branch_distance_sum(&node_mins, &node_maxs, &queries).unwrap(), 25);
        assert_eq!(
            subtract_first_distance_sum(&node_mins, &node_maxs, &queries).unwrap(),
            25
        );
    }

    #[test]
    fn rejects_bad_lengths() {
        let node_mins = [0, 1, 2];
        let node_maxs = [0, 1, 2];
        let queries = [0, 1, 2, 3];
        assert_eq!(
            old_distance_sum(&node_mins, &node_maxs, &queries),
            Err(ClimateError::InvalidInputLength)
        );
    }
}
