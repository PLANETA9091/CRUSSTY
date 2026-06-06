pub const SUMMARY_FIELDS: usize = 5;

pub const TYPE_TRANSIENT: u8 = 0;
pub const TYPE_FUTURE: u8 = 1;
pub const TYPE_FORCED: u8 = 2;
pub const TYPE_PORTAL: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TicketPackError {
    LengthMismatch,
    InvalidTicketType,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TicketPackSummary {
    pub persistent_count: u64,
    pub level_sum: u64,
    pub position_checksum: u64,
    pub consume_value: u64,
    pub sink: u64,
}

pub fn pack_summary(
    positions: &[i64],
    ticket_types: &[u8],
    ticket_levels: &[i32],
    tickets_per_chunk: usize,
    iterations: usize,
) -> Result<TicketPackSummary, TicketPackError> {
    let expected_ticket_count = positions
        .len()
        .checked_mul(tickets_per_chunk)
        .ok_or(TicketPackError::LengthMismatch)?;
    if ticket_types.len() != expected_ticket_count || ticket_levels.len() != expected_ticket_count {
        return Err(TicketPackError::LengthMismatch);
    }

    let mut summary = scan_once(positions, ticket_types, ticket_levels, tickets_per_chunk)?;
    if iterations == 0 {
        return Ok(summary);
    }

    for _ in 0..iterations {
        let value = consume_once(
            positions,
            ticket_types,
            ticket_levels,
            tickets_per_chunk,
            summary.persistent_count,
        )?;
        summary.consume_value = value;
        summary.sink = summary.sink.wrapping_add(value);
    }

    Ok(summary)
}

fn scan_once(
    positions: &[i64],
    ticket_types: &[u8],
    ticket_levels: &[i32],
    tickets_per_chunk: usize,
) -> Result<TicketPackSummary, TicketPackError> {
    let mut summary = TicketPackSummary::default();

    for (chunk_index, &position) in positions.iter().enumerate() {
        let base = chunk_index * tickets_per_chunk;
        for ticket_index in 0..tickets_per_chunk {
            let flat_index = base + ticket_index;
            if !is_persistent_type(ticket_types[flat_index])? {
                continue;
            }

            let level = ticket_levels[flat_index];
            let entry_index = summary.persistent_count;
            summary.persistent_count += 1;
            summary.level_sum = summary.level_sum.wrapping_add(level as i64 as u64);
            summary.position_checksum = mix64(
                summary.position_checksum
                    ^ (position as u64)
                    ^ ((level as i64 as u64).rotate_left(17))
                    ^ entry_index.wrapping_mul(0x9E37_79B9_7F4A_7C15),
            );
        }
    }

    Ok(summary)
}

fn consume_once(
    positions: &[i64],
    ticket_types: &[u8],
    ticket_levels: &[i32],
    tickets_per_chunk: usize,
    persistent_count: u64,
) -> Result<u64, TicketPackError> {
    let mut value = persistent_count;
    for (chunk_index, &position) in positions.iter().enumerate() {
        let base = chunk_index * tickets_per_chunk;
        for ticket_index in 0..tickets_per_chunk {
            let flat_index = base + ticket_index;
            if !is_persistent_type(ticket_types[flat_index])? {
                continue;
            }

            value = value.wrapping_mul(31).wrapping_add(position as u64);
            value = value
                .wrapping_mul(31)
                .wrapping_add(ticket_levels[flat_index] as i64 as u64);
        }
    }
    Ok(value)
}

#[inline]
fn is_persistent_type(ticket_type: u8) -> Result<bool, TicketPackError> {
    match ticket_type {
        TYPE_TRANSIENT | TYPE_FUTURE => Ok(false),
        TYPE_FORCED | TYPE_PORTAL => Ok(true),
        _ => Err(TicketPackError::InvalidTicketType),
    }
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
    fn summarizes_persistent_tickets_in_chunk_order() {
        let positions = [10, -5];
        let ticket_types = [
            TYPE_FORCED,
            TYPE_TRANSIENT,
            TYPE_PORTAL,
            TYPE_FUTURE,
            TYPE_FORCED,
            TYPE_TRANSIENT,
        ];
        let ticket_levels = [1, 2, 3, 4, 5, 6];

        let summary = pack_summary(&positions, &ticket_types, &ticket_levels, 3, 2).unwrap();
        assert_eq!(summary.persistent_count, 3);
        assert_eq!(summary.level_sum, 9);
        assert_eq!(summary.consume_value, reference_consume(&[(10, 1), (10, 3), (-5, 5)]));
        assert_eq!(summary.sink, summary.consume_value.wrapping_mul(2));
        assert_ne!(summary.position_checksum, 0);
    }

    #[test]
    fn rejects_bad_shapes_and_types() {
        assert_eq!(
            pack_summary(&[1], &[TYPE_FORCED], &[], 1, 1),
            Err(TicketPackError::LengthMismatch)
        );
        assert_eq!(
            pack_summary(&[1], &[99], &[1], 1, 1),
            Err(TicketPackError::InvalidTicketType)
        );
    }

    #[test]
    fn zero_iterations_keeps_one_pass_summary_without_sink() {
        let summary = pack_summary(&[1], &[TYPE_FORCED, TYPE_PORTAL], &[7, 9], 2, 0).unwrap();
        assert_eq!(summary.persistent_count, 2);
        assert_eq!(summary.level_sum, 16);
        assert_eq!(summary.consume_value, 0);
        assert_eq!(summary.sink, 0);
    }

    #[test]
    fn random_cases_match_reference_model() {
        let mut state = 0xA51C_71C7_2026_0512u64;
        for chunks in [0usize, 1, 3, 17, 64] {
            for tickets_per_chunk in [0usize, 1, 2, 8] {
                let ticket_count = chunks * tickets_per_chunk;
                let mut positions = Vec::with_capacity(chunks);
                let mut ticket_types = Vec::with_capacity(ticket_count);
                let mut ticket_levels = Vec::with_capacity(ticket_count);

                for _ in 0..chunks {
                    state = next(state);
                    let x = ((state >> 17) as i32 % 8_000) - 4_000;
                    state = next(state);
                    let z = ((state >> 21) as i32 % 8_000) - 4_000;
                    positions.push((x as u32 as u64 | ((z as u64) << 32)) as i64);

                    for ticket_index in 0..tickets_per_chunk {
                        state = next(state);
                        let ticket_type = match (state as usize + ticket_index) & 3 {
                            0 => TYPE_TRANSIENT,
                            1 => TYPE_FUTURE,
                            2 => TYPE_FORCED,
                            _ => TYPE_PORTAL,
                        };
                        ticket_types.push(ticket_type);
                        ticket_levels.push(((state >> 32) as i32) & 63);
                    }
                }

                let actual = pack_summary(
                    &positions,
                    &ticket_types,
                    &ticket_levels,
                    tickets_per_chunk,
                    3,
                )
                .unwrap();
                let expected = reference_summary(
                    &positions,
                    &ticket_types,
                    &ticket_levels,
                    tickets_per_chunk,
                    3,
                );
                assert_eq!(actual, expected);
            }
        }
    }

    fn reference_summary(
        positions: &[i64],
        ticket_types: &[u8],
        ticket_levels: &[i32],
        tickets_per_chunk: usize,
        iterations: usize,
    ) -> TicketPackSummary {
        let mut entries = Vec::new();
        for (chunk_index, &position) in positions.iter().enumerate() {
            let base = chunk_index * tickets_per_chunk;
            for ticket_index in 0..tickets_per_chunk {
                let flat_index = base + ticket_index;
                if matches!(ticket_types[flat_index], TYPE_FORCED | TYPE_PORTAL) {
                    entries.push((position, ticket_levels[flat_index]));
                }
            }
        }

        let mut summary = TicketPackSummary::default();
        for (entry_index, &(position, level)) in entries.iter().enumerate() {
            summary.persistent_count += 1;
            summary.level_sum = summary.level_sum.wrapping_add(level as i64 as u64);
            summary.position_checksum = mix64(
                summary.position_checksum
                    ^ (position as u64)
                    ^ ((level as i64 as u64).rotate_left(17))
                    ^ (entry_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
            );
        }

        for _ in 0..iterations {
            let value = reference_consume(&entries);
            summary.consume_value = value;
            summary.sink = summary.sink.wrapping_add(value);
        }
        summary
    }

    fn reference_consume(entries: &[(i64, i32)]) -> u64 {
        let mut value = entries.len() as u64;
        for &(position, level) in entries {
            value = value.wrapping_mul(31).wrapping_add(position as u64);
            value = value.wrapping_mul(31).wrapping_add(level as i64 as u64);
        }
        value
    }

    fn next(value: u64) -> u64 {
        value.wrapping_mul(6364136223846793005).wrapping_add(1)
    }
}
