pub const PARAMETER_COUNT: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClimateError {
    InvalidInputLength,
    OutputTooSmall(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BestMatch {
    pub index: usize,
    pub score: i64,
}

#[inline]
pub fn parameter_distance(min: i64, max: i64, value: i64) -> i64 {
    if value > max {
        value.wrapping_sub(max)
    } else if value < min {
        min.wrapping_sub(value)
    } else {
        0
    }
}

#[inline]
pub fn node_distance_sum(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
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
            sum = sum.wrapping_add(node_score(node_min, node_max, query));
        }
    }

    Ok(sum)
}

#[inline]
pub fn node_distance_sum_batch(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
    dst: &mut [i64],
) -> Result<usize, ClimateError> {
    if dst.is_empty() {
        return Err(ClimateError::OutputTooSmall(1));
    }

    let sum = node_distance_sum(node_mins, node_maxs, queries)?;
    dst[0] = sum;
    Ok(1)
}

#[inline]
pub fn node_best_match_batch(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateError> {
    if node_mins.len() != node_maxs.len()
        || node_mins.is_empty()
        || node_mins.len() % PARAMETER_COUNT != 0
        || queries.len() % PARAMETER_COUNT != 0
    {
        return Err(ClimateError::InvalidInputLength);
    }

    let query_count = queries.len() / PARAMETER_COUNT;
    if best_indices.len() < query_count || best_scores.len() < query_count {
        return Err(ClimateError::OutputTooSmall(query_count));
    }

    for (query_index, query) in queries.chunks_exact(PARAMETER_COUNT).enumerate() {
        let mut best = BestMatch { index: 0, score: i64::MAX };
        for (node_index, (node_min, node_max)) in node_mins
            .chunks_exact(PARAMETER_COUNT)
            .zip(node_maxs.chunks_exact(PARAMETER_COUNT))
            .enumerate()
        {
            let score = if best.score > 0 {
                node_score_bounded(node_min, node_max, query, best.score)
            } else {
                node_score(node_min, node_max, query)
            };
            if score < best.score || (score == best.score && node_index < best.index) {
                best = BestMatch { index: node_index, score };
            }
        }
        best_indices[query_index] = best.index as i32;
        best_scores[query_index] = best.score;
    }

    Ok(query_count)
}

#[inline]
pub fn node_best_match_unique_batch(
    node_mins: &[i64],
    node_maxs: &[i64],
    queries: &[i64],
    best_indices: &mut [i32],
    best_scores: &mut [i64],
) -> Result<usize, ClimateError> {
    if node_mins.len() != node_maxs.len()
        || node_mins.is_empty()
        || node_mins.len() % PARAMETER_COUNT != 0
        || queries.len() % PARAMETER_COUNT != 0
    {
        return Err(ClimateError::InvalidInputLength);
    }

    let query_count = queries.len() / PARAMETER_COUNT;
    if best_indices.len() < query_count || best_scores.len() < query_count {
        return Err(ClimateError::OutputTooSmall(query_count));
    }

    for (query_index, query) in queries.chunks_exact(PARAMETER_COUNT).enumerate() {
        let mut best = BestMatch { index: 0, score: i64::MAX };
        let mut tied_best = false;
        for (node_index, (node_min, node_max)) in node_mins
            .chunks_exact(PARAMETER_COUNT)
            .zip(node_maxs.chunks_exact(PARAMETER_COUNT))
            .enumerate()
        {
            let score = if best.score > 0 {
                node_score_bounded(node_min, node_max, query, best.score)
            } else {
                node_score(node_min, node_max, query)
            };
            if score < best.score {
                best = BestMatch { index: node_index, score };
                tied_best = false;
            } else if score == best.score {
                tied_best = true;
            }
        }
        best_indices[query_index] = if tied_best { -1 } else { best.index as i32 };
        best_scores[query_index] = best.score;
    }

    Ok(query_count)
}

#[inline]
fn node_score(node_mins: &[i64], node_maxs: &[i64], query: &[i64]) -> i64 {
    let mut node_sum = 0i64;
    for parameter in 0..PARAMETER_COUNT {
        let distance = parameter_distance(node_mins[parameter], node_maxs[parameter], query[parameter]);
        node_sum = node_sum.wrapping_add(distance.wrapping_mul(distance));
    }
    node_sum
}

#[inline]
fn node_score_bounded(node_mins: &[i64], node_maxs: &[i64], query: &[i64], limit: i64) -> i64 {
    let mut node_sum = 0i64;
    for parameter in 0..PARAMETER_COUNT {
        let distance = parameter_distance(node_mins[parameter], node_maxs[parameter], query[parameter]);
        node_sum = node_sum.wrapping_add(distance.wrapping_mul(distance));
        if node_sum >= limit {
            return limit;
        }
    }
    node_sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_distance_matches_reference() {
        assert_eq!(parameter_distance(10, 20, 5), 5);
        assert_eq!(parameter_distance(10, 20, 10), 0);
        assert_eq!(parameter_distance(10, 20, 15), 0);
        assert_eq!(parameter_distance(10, 20, 25), 5);
    }

    #[test]
    fn node_distance_sum_matches_reference() {
        let node_mins = [0, 10, 20, 30, 40, 50, 60];
        let node_maxs = [5, 15, 25, 35, 45, 55, 65];
        let queries = [3, 12, 22, 33, 43, 52, 70];
        assert_eq!(node_distance_sum(&node_mins, &node_maxs, &queries).unwrap(), 25);
    }

    #[test]
    fn node_distance_sum_rejects_bad_lengths() {
        let node_mins = [0, 1, 2];
        let node_maxs = [0, 1, 2];
        let queries = [0, 1, 2, 3];
        assert_eq!(node_distance_sum(&node_mins, &node_maxs, &queries), Err(ClimateError::InvalidInputLength));
    }

    #[test]
    fn node_distance_sum_batch_writes_result() {
        let node_mins = [0, 10, 20, 30, 40, 50, 60];
        let node_maxs = [5, 15, 25, 35, 45, 55, 65];
        let queries = [3, 12, 22, 33, 43, 52, 70];
        let mut dst = [0i64; 1];
        assert_eq!(node_distance_sum_batch(&node_mins, &node_maxs, &queries, &mut dst).unwrap(), 1);
        assert_eq!(dst[0], 25);
    }

    #[test]
    fn node_best_match_batch_picks_lowest_index_on_tie() {
        let node_mins = [0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 10];
        let node_maxs = [0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 10];
        let queries = [5, 5, 5, 5, 5, 5, 5];
        let mut indices = [0i32; 1];
        let mut scores = [0i64; 1];
        assert_eq!(
            node_best_match_batch(&node_mins, &node_maxs, &queries, &mut indices, &mut scores).unwrap(),
            1
        );
        assert_eq!(indices[0], 0);
        assert_eq!(scores[0], 175);
    }

    #[test]
    fn node_best_match_batch_writes_each_query() {
        let node_mins = [0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 10];
        let node_maxs = [0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 10];
        let queries = [1, 1, 1, 1, 1, 1, 1, 9, 9, 9, 9, 9, 9, 9];
        let mut indices = [0i32; 2];
        let mut scores = [0i64; 2];
        assert_eq!(
            node_best_match_batch(&node_mins, &node_maxs, &queries, &mut indices, &mut scores).unwrap(),
            2
        );
        assert_eq!(indices, [0, 1]);
        assert_eq!(scores, [7, 7]);
    }

    #[test]
    fn node_best_match_unique_batch_marks_ties_for_fallback() {
        let node_mins = [0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 10];
        let node_maxs = [0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 10];
        let queries = [5, 5, 5, 5, 5, 5, 5];
        let mut indices = [0i32; 1];
        let mut scores = [0i64; 1];
        assert_eq!(
            node_best_match_unique_batch(&node_mins, &node_maxs, &queries, &mut indices, &mut scores).unwrap(),
            1
        );
        assert_eq!(indices[0], -1);
        assert_eq!(scores[0], 175);
    }

    #[test]
    fn node_best_match_unique_batch_keeps_unique_best_index() {
        let node_mins = [0, 0, 0, 0, 0, 0, 0, 20, 20, 20, 20, 20, 20, 20];
        let node_maxs = [0, 0, 0, 0, 0, 0, 0, 20, 20, 20, 20, 20, 20, 20];
        let queries = [18, 18, 18, 18, 18, 18, 18];
        let mut indices = [0i32; 1];
        let mut scores = [0i64; 1];
        assert_eq!(
            node_best_match_unique_batch(&node_mins, &node_maxs, &queries, &mut indices, &mut scores).unwrap(),
            1
        );
        assert_eq!(indices[0], 1);
        assert_eq!(scores[0], 28);
    }

    #[test]
    fn node_best_match_batch_rejects_empty_nodes() {
        let queries = [0, 0, 0, 0, 0, 0, 0];
        let mut indices = [0i32; 1];
        let mut scores = [0i64; 1];
        assert_eq!(
            node_best_match_batch(&[], &[], &queries, &mut indices, &mut scores),
            Err(ClimateError::InvalidInputLength)
        );
    }

    #[test]
    fn node_best_match_batch_rejects_small_output() {
        let node_mins = [0, 0, 0, 0, 0, 0, 0];
        let node_maxs = [0, 0, 0, 0, 0, 0, 0];
        let queries = [0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1];
        let mut indices = [0i32; 1];
        let mut scores = [0i64; 2];
        assert_eq!(
            node_best_match_batch(&node_mins, &node_maxs, &queries, &mut indices, &mut scores),
            Err(ClimateError::OutputTooSmall(2))
        );
    }
}
