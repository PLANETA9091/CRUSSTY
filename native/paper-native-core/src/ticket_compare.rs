#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TicketCompareError {
    LengthMismatch,
    InvalidIndex,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TicketCompareSummary {
    pub compare_sum: i64,
    pub negative_count: u64,
    pub zero_count: u64,
    pub positive_count: u64,
}

#[derive(Clone, Copy, Debug)]
struct TicketView {
    level: i32,
    type_id: i64,
    has_identifier_comparator: bool,
    identifier: i32,
}

pub fn compare_indexed_batch(
    levels: &[i32],
    type_ids: &[i64],
    has_identifier_comparators: &[u8],
    identifiers: &[i32],
    left_indices: &[i32],
    right_indices: &[i32],
    iterations: usize,
) -> Result<TicketCompareSummary, TicketCompareError> {
    let ticket_count = levels.len();
    if type_ids.len() != ticket_count
        || has_identifier_comparators.len() != ticket_count
        || identifiers.len() != ticket_count
        || left_indices.len() != right_indices.len()
    {
        return Err(TicketCompareError::LengthMismatch);
    }
    if left_indices.is_empty() && iterations != 0 {
        return Err(TicketCompareError::LengthMismatch);
    }

    let mut summary = TicketCompareSummary::default();
    for iteration in 0..iterations {
        let op_index = iteration % left_indices.len();
        let left_index = to_index(left_indices[op_index], ticket_count)?;
        let right_index = to_index(right_indices[op_index], ticket_count)?;
        let compared = compare_ticket(
            ticket_at(
                levels,
                type_ids,
                has_identifier_comparators,
                identifiers,
                left_index,
            ),
            ticket_at(
                levels,
                type_ids,
                has_identifier_comparators,
                identifiers,
                right_index,
            ),
        );
        summary.compare_sum = summary.compare_sum.wrapping_add(compared as i64);
        match compared.cmp(&0) {
            std::cmp::Ordering::Less => summary.negative_count += 1,
            std::cmp::Ordering::Equal => summary.zero_count += 1,
            std::cmp::Ordering::Greater => summary.positive_count += 1,
        }
    }

    Ok(summary)
}

#[inline]
fn ticket_at(
    levels: &[i32],
    type_ids: &[i64],
    has_identifier_comparators: &[u8],
    identifiers: &[i32],
    index: usize,
) -> TicketView {
    TicketView {
        level: levels[index],
        type_id: type_ids[index],
        has_identifier_comparator: has_identifier_comparators[index] != 0,
        identifier: identifiers[index],
    }
}

#[inline]
fn to_index(index: i32, len: usize) -> Result<usize, TicketCompareError> {
    if index < 0 {
        return Err(TicketCompareError::InvalidIndex);
    }
    let index = index as usize;
    if index >= len {
        return Err(TicketCompareError::InvalidIndex);
    }
    Ok(index)
}

#[inline]
fn compare_ticket(left: TicketView, right: TicketView) -> i32 {
    let level_compare = ordering_to_i32(left.level.cmp(&right.level));
    if level_compare != 0 {
        return level_compare;
    }

    let type_compare = ordering_to_i32(left.type_id.cmp(&right.type_id));
    if type_compare != 0 {
        return type_compare;
    }

    if left.has_identifier_comparator {
        ordering_to_i32(left.identifier.cmp(&right.identifier))
    } else {
        0
    }
}

#[inline]
fn ordering_to_i32(ordering: std::cmp::Ordering) -> i32 {
    match ordering {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_level_before_type_and_identifier() {
        let levels = [10, 20, 10, 10];
        let type_ids = [1, 0, 2, 2];
        let has_comparators = [0, 0, 1, 1];
        let identifiers = [0, 0, 30, 20];
        let left = [0, 0, 2, 3, 2];
        let right = [1, 2, 3, 2, 2];

        let summary = compare_indexed_batch(
            &levels,
            &type_ids,
            &has_comparators,
            &identifiers,
            &left,
            &right,
            left.len(),
        )
        .unwrap();

        assert_eq!(summary.negative_count, 3);
        assert_eq!(summary.zero_count, 1);
        assert_eq!(summary.positive_count, 1);
        assert_eq!(summary.compare_sum, -2);
    }

    #[test]
    fn comparatorless_equal_type_ignores_identifier() {
        let levels = [10, 10];
        let type_ids = [1, 1];
        let has_comparators = [0, 0];
        let identifiers = [1, 999];
        let summary = compare_indexed_batch(
            &levels,
            &type_ids,
            &has_comparators,
            &identifiers,
            &[0],
            &[1],
            1,
        )
        .unwrap();
        assert_eq!(summary.zero_count, 1);
        assert_eq!(summary.compare_sum, 0);
    }

    #[test]
    fn rejects_bad_shapes_and_indices() {
        assert_eq!(
            compare_indexed_batch(&[1], &[], &[0], &[0], &[], &[], 0),
            Err(TicketCompareError::LengthMismatch)
        );
        assert_eq!(
            compare_indexed_batch(&[1], &[1], &[0], &[0], &[0], &[1], 1),
            Err(TicketCompareError::InvalidIndex)
        );
        assert_eq!(
            compare_indexed_batch(&[1], &[1], &[0], &[0], &[], &[], 1),
            Err(TicketCompareError::LengthMismatch)
        );
    }

    #[test]
    fn random_cases_match_reference_model() {
        let mut state = 0x71C4_E720_2605_12u64;
        for ticket_count in [1usize, 4, 32, 257] {
            let mut levels = Vec::with_capacity(ticket_count);
            let mut type_ids = Vec::with_capacity(ticket_count);
            let mut has_comparators = Vec::with_capacity(ticket_count);
            let mut identifiers = Vec::with_capacity(ticket_count);
            for _ in 0..ticket_count {
                state = next(state);
                levels.push(20 + ((state >> 17) as i32 & 31));
                state = next(state);
                let type_id = ((state >> 23) % 6) as i64;
                type_ids.push(type_id);
                has_comparators.push(if type_id == 2 || type_id == 4 { 1 } else { 0 });
                state = next(state);
                identifiers.push((state >> 32) as i32);
            }

            let mut left = Vec::with_capacity(4096);
            let mut right = Vec::with_capacity(4096);
            for _ in 0..4096 {
                state = next(state);
                left.push(((state >> 16) as usize % ticket_count) as i32);
                state = next(state);
                right.push(((state >> 16) as usize % ticket_count) as i32);
            }

            let actual = compare_indexed_batch(
                &levels,
                &type_ids,
                &has_comparators,
                &identifiers,
                &left,
                &right,
                1 << 15,
            )
            .unwrap();
            let expected = reference_summary(
                &levels,
                &type_ids,
                &has_comparators,
                &identifiers,
                &left,
                &right,
                1 << 15,
            );
            assert_eq!(actual, expected);
        }
    }

    fn reference_summary(
        levels: &[i32],
        type_ids: &[i64],
        has_comparators: &[u8],
        identifiers: &[i32],
        left: &[i32],
        right: &[i32],
        iterations: usize,
    ) -> TicketCompareSummary {
        let mut summary = TicketCompareSummary::default();
        for iteration in 0..iterations {
            let op_index = iteration % left.len();
            let left_index = left[op_index] as usize;
            let right_index = right[op_index] as usize;
            let compared = reference_compare(
                levels[left_index],
                type_ids[left_index],
                has_comparators[left_index] != 0,
                identifiers[left_index],
                levels[right_index],
                type_ids[right_index],
                identifiers[right_index],
            );
            summary.compare_sum = summary.compare_sum.wrapping_add(compared as i64);
            if compared < 0 {
                summary.negative_count += 1;
            } else if compared == 0 {
                summary.zero_count += 1;
            } else {
                summary.positive_count += 1;
            }
        }
        summary
    }

    fn reference_compare(
        left_level: i32,
        left_type_id: i64,
        left_has_comparator: bool,
        left_identifier: i32,
        right_level: i32,
        right_type_id: i64,
        right_identifier: i32,
    ) -> i32 {
        if left_level != right_level {
            return if left_level < right_level { -1 } else { 1 };
        }
        if left_type_id != right_type_id {
            return if left_type_id < right_type_id { -1 } else { 1 };
        }
        if left_has_comparator && left_identifier != right_identifier {
            return if left_identifier < right_identifier {
                -1
            } else {
                1
            };
        }
        0
    }

    fn next(value: u64) -> u64 {
        value.wrapping_mul(6364136223846793005).wrapping_add(1)
    }
}
