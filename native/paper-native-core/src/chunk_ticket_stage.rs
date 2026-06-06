pub const SUMMARY_FIELDS: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChunkTicketStageError {
    LengthMismatch,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChunkTicketStageSummary {
    pub get_sum: i64,
    pub mutation_sum: i64,
    pub final_size: u64,
    pub state_checksum: u64,
}

pub fn run_batch(
    query_keys: &[i64],
    staged_keys: &[i64],
    staged_values: &[i8],
    mutation_keys: &[i64],
    get_iterations: usize,
    mutation_iterations: usize,
) -> Result<ChunkTicketStageSummary, ChunkTicketStageError> {
    if staged_keys.len() != staged_values.len() {
        return Err(ChunkTicketStageError::LengthMismatch);
    }
    if (query_keys.is_empty() && get_iterations != 0)
        || (staged_keys.is_empty() && mutation_iterations != 0)
        || (mutation_keys.is_empty() && mutation_iterations != 0)
    {
        return Err(ChunkTicketStageError::LengthMismatch);
    }

    let mut map = LongByteMap::with_expected(staged_keys.len());
    for index in 0..staged_keys.len() {
        map.put(staged_keys[index], staged_values[index]);
    }

    let get_sum = run_get_sweep(&map, query_keys, get_iterations);
    let mutation_sum = run_mutation_churn(
        &mut map,
        staged_keys,
        staged_values,
        mutation_keys,
        mutation_iterations,
    );

    Ok(ChunkTicketStageSummary {
        get_sum,
        mutation_sum,
        final_size: map.len() as u64,
        state_checksum: state_checksum(&map, staged_keys),
    })
}

fn run_get_sweep(map: &LongByteMap, query_keys: &[i64], iterations: usize) -> i64 {
    let mut sum = 0i64;
    for iteration in 0..iterations {
        let offset = (iteration * 53) % query_keys.len();
        for index in 0..query_keys.len() {
            sum += map.get(query_keys[(index + offset) % query_keys.len()]) as i64;
        }
    }
    sum
}

fn run_mutation_churn(
    map: &mut LongByteMap,
    staged_keys: &[i64],
    staged_values: &[i8],
    mutation_keys: &[i64],
    iterations: usize,
) -> i64 {
    let mut sum = 0i64;
    for iteration in 0..iterations {
        let staged_index = (iteration * 31) % staged_keys.len();
        let mutation_index = (iteration * 17) % mutation_keys.len();
        let remove_key = staged_keys[staged_index];
        let add_key = mutation_keys[mutation_index];
        let value = staged_values[staged_index];

        sum += map.remove(remove_key) as i64;
        sum += map.put(add_key, value) as i64;
        sum += map.get(add_key) as i64;
        sum += map.put(remove_key, value) as i64;
        sum += map.remove(add_key) as i64;
    }
    sum + map.len() as i64
}

fn state_checksum(map: &LongByteMap, staged_keys: &[i64]) -> u64 {
    let mut checksum = 0u64;
    for (index, &key) in staged_keys.iter().enumerate() {
        checksum = mix64(
            checksum
                ^ (key as u64)
                ^ ((map.get(key) as i64 as u64) << 32)
                ^ ((index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
        );
    }
    checksum
}

#[derive(Clone, Debug)]
struct LongByteMap {
    keys: Vec<i64>,
    values: Vec<i8>,
    states: Vec<u8>,
    mask: usize,
    size: usize,
    deleted: usize,
}

impl LongByteMap {
    fn with_expected(expected: usize) -> Self {
        let mut capacity = 16usize;
        let wanted = expected.max(1);
        while capacity * 3 / 4 < wanted {
            capacity <<= 1;
        }
        Self {
            keys: vec![0; capacity],
            values: vec![0; capacity],
            states: vec![0; capacity],
            mask: capacity - 1,
            size: 0,
            deleted: 0,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.size
    }

    fn get(&self, key: i64) -> i8 {
        let mut slot = self.slot(key);
        loop {
            match self.states[slot] {
                0 => return 0,
                1 if self.keys[slot] == key => return self.values[slot],
                _ => slot = (slot + 1) & self.mask,
            }
        }
    }

    fn put(&mut self, key: i64, value: i8) -> i8 {
        if (self.size + self.deleted + 1) * 4 >= self.keys.len() * 3 {
            self.rehash(self.keys.len() * 2);
        }

        let mut slot = self.slot(key);
        let mut first_deleted = usize::MAX;
        loop {
            match self.states[slot] {
                0 => {
                    let insert_slot = if first_deleted == usize::MAX {
                        slot
                    } else {
                        self.deleted -= 1;
                        first_deleted
                    };
                    self.keys[insert_slot] = key;
                    self.values[insert_slot] = value;
                    self.states[insert_slot] = 1;
                    self.size += 1;
                    return 0;
                }
                1 if self.keys[slot] == key => {
                    let old = self.values[slot];
                    self.values[slot] = value;
                    return old;
                }
                2 if first_deleted == usize::MAX => first_deleted = slot,
                _ => {}
            }
            slot = (slot + 1) & self.mask;
        }
    }

    fn remove(&mut self, key: i64) -> i8 {
        let mut slot = self.slot(key);
        loop {
            match self.states[slot] {
                0 => return 0,
                1 if self.keys[slot] == key => {
                    self.states[slot] = 2;
                    self.size -= 1;
                    self.deleted += 1;
                    return self.values[slot];
                }
                _ => slot = (slot + 1) & self.mask,
            }
        }
    }

    fn rehash(&mut self, new_capacity: usize) {
        let old_keys = std::mem::replace(&mut self.keys, vec![0; new_capacity]);
        let old_values = std::mem::replace(&mut self.values, vec![0; new_capacity]);
        let old_states = std::mem::replace(&mut self.states, vec![0; new_capacity]);
        self.mask = new_capacity - 1;
        self.size = 0;
        self.deleted = 0;

        for index in 0..old_keys.len() {
            if old_states[index] == 1 {
                self.put(old_keys[index], old_values[index]);
            }
        }
    }

    #[inline]
    fn slot(&self, key: i64) -> usize {
        (mix64(key as u64) as usize) & self.mask
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
    use std::collections::HashMap;

    #[test]
    fn run_batch_matches_reference_model() {
        let query_keys = [-2, -1, 0, 1, 2, 3, 4];
        let staged_keys = [-1, 1, 3];
        let staged_values = [2, 4, 5];
        let mutation_keys = [10, 11, 12, 10];

        let actual = run_batch(
            &query_keys,
            &staged_keys,
            &staged_values,
            &mutation_keys,
            13,
            101,
        )
        .unwrap();
        let expected = reference_run(
            &query_keys,
            &staged_keys,
            &staged_values,
            &mutation_keys,
            13,
            101,
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn random_cases_match_reference_model() {
        let mut state = 0xC4A1_7E75_2605_12u64;
        for size in [1usize, 16, 257, 4096] {
            let mut query_keys = Vec::with_capacity(size * 2);
            let mut staged_keys = Vec::with_capacity(size);
            let mut staged_values = Vec::with_capacity(size);
            let mut mutation_keys = Vec::with_capacity(size);

            for index in 0..size * 2 {
                query_keys.push((index as i64) * 37 - size as i64);
            }
            for index in 0..size {
                state = next(state);
                staged_keys.push(1_000_000 + index as i64 * 97 + ((state >> 17) as i64 & 63));
                staged_values.push(1 + ((state >> 17) % 5) as i8);
                mutation_keys.push(1_000_000 + index as i64 * 97);
            }

            let actual = run_batch(
                &query_keys,
                &staged_keys,
                &staged_values,
                &mutation_keys,
                128,
                2048,
            )
            .unwrap();
            let expected = reference_run(
                &query_keys,
                &staged_keys,
                &staged_values,
                &mutation_keys,
                128,
                2048,
            );
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn rejects_bad_shapes() {
        assert_eq!(
            run_batch(&[1], &[1], &[], &[2], 1, 1),
            Err(ChunkTicketStageError::LengthMismatch)
        );
        assert_eq!(
            run_batch(&[], &[1], &[1], &[2], 1, 0),
            Err(ChunkTicketStageError::LengthMismatch)
        );
        assert_eq!(
            run_batch(&[1], &[1], &[1], &[], 0, 1),
            Err(ChunkTicketStageError::LengthMismatch)
        );
    }

    fn reference_run(
        query_keys: &[i64],
        staged_keys: &[i64],
        staged_values: &[i8],
        mutation_keys: &[i64],
        get_iterations: usize,
        mutation_iterations: usize,
    ) -> ChunkTicketStageSummary {
        let mut map = HashMap::new();
        for index in 0..staged_keys.len() {
            map.insert(staged_keys[index], staged_values[index]);
        }

        let mut get_sum = 0i64;
        for iteration in 0..get_iterations {
            let offset = (iteration * 53) % query_keys.len();
            for index in 0..query_keys.len() {
                get_sum += *map
                    .get(&query_keys[(index + offset) % query_keys.len()])
                    .unwrap_or(&0) as i64;
            }
        }

        let mut mutation_sum = 0i64;
        for iteration in 0..mutation_iterations {
            let staged_index = (iteration * 31) % staged_keys.len();
            let mutation_index = (iteration * 17) % mutation_keys.len();
            let remove_key = staged_keys[staged_index];
            let add_key = mutation_keys[mutation_index];
            let value = staged_values[staged_index];

            mutation_sum += map.remove(&remove_key).unwrap_or(0) as i64;
            mutation_sum += map.insert(add_key, value).unwrap_or(0) as i64;
            mutation_sum += *map.get(&add_key).unwrap_or(&0) as i64;
            mutation_sum += map.insert(remove_key, value).unwrap_or(0) as i64;
            mutation_sum += map.remove(&add_key).unwrap_or(0) as i64;
        }
        mutation_sum += map.len() as i64;

        let mut checksum = 0u64;
        for (index, &key) in staged_keys.iter().enumerate() {
            checksum = mix64(
                checksum
                    ^ (key as u64)
                    ^ ((*map.get(&key).unwrap_or(&0) as i64 as u64) << 32)
                    ^ ((index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
            );
        }

        ChunkTicketStageSummary {
            get_sum,
            mutation_sum,
            final_size: map.len() as u64,
            state_checksum: checksum,
        }
    }

    fn next(value: u64) -> u64 {
        value.wrapping_mul(6364136223846793005).wrapping_add(1)
    }
}
