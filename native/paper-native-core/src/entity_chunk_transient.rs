pub const SUMMARY_FIELDS: usize = 4;

const TICKET_GAMMA: i64 = 0x9E37_79B9_7F4A_7C15u64 as i64;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EntityChunkTransientSummary {
    pub count: u64,
    pub value: i64,
    pub non_transient_count: u64,
    pub last_ticket: i64,
}

pub fn old_mixed_summary(iterations: usize, non_transient_mask: i32, thread_id: i64) -> EntityChunkTransientSummary {
    run_summary(iterations, non_transient_mask, thread_id)
}

pub fn new_mixed_summary(iterations: usize, non_transient_mask: i32, thread_id: i64) -> EntityChunkTransientSummary {
    run_summary(iterations, non_transient_mask, thread_id)
}

fn run_summary(iterations: usize, non_transient_mask: i32, thread_id: i64) -> EntityChunkTransientSummary {
    if iterations == 0 {
        return EntityChunkTransientSummary::default();
    }

    let mut value = 0i64;
    let mut non_transient_count = 0u64;
    let mut last_ticket = 0i64;

    for i in 0..iterations {
        let transient_chunk = ((i as i32) & non_transient_mask) != 0;
        let ticket = (i as i64).wrapping_mul(TICKET_GAMMA);
        last_ticket = ticket;
        let item = if transient_chunk {
            ticket
        } else {
            non_transient_count = non_transient_count.wrapping_add(1);
            ticket ^ thread_id
        };
        value = value.wrapping_add(item);
    }

    EntityChunkTransientSummary {
        count: iterations as u64,
        value,
        non_transient_count,
        last_ticket,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_new_match() {
        assert_eq!(
            old_mixed_summary(4096, 0x0F, 17),
            new_mixed_summary(4096, 0x0F, 17)
        );
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(
            old_mixed_summary(0, 0x0F, 17),
            EntityChunkTransientSummary::default()
        );
    }
}
