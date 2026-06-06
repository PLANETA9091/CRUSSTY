const EMPTY_KEY: i32 = i32::MIN;
pub const MAX_LINEAR_TRANSITION_REMOVE_LIMIT: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceListError {
    LengthMismatch,
    InvalidOperation,
    InvalidValue,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReferenceListRunSummary {
    pub add_true: u64,
    pub remove_true: u64,
    pub contains_true: u64,
    pub false_count: u64,
    pub final_size: u64,
    pub event_checksum: u64,
    pub order_checksum: u64,
}

impl ReferenceListRunSummary {
    #[inline]
    fn record(&mut self, op: u8, value: i32, result: bool, size: usize) {
        match (op, result) {
            (0, true) => self.add_true += 1,
            (1, true) => self.remove_true += 1,
            (2, true) => self.contains_true += 1,
            (_, false) => self.false_count += 1,
            _ => {}
        }

        let op_tag = match op {
            0 => 0x9E37_79B9_7F4A_7C15u64,
            1 => 0xC2B2_AE3D_27D4_EB4Fu64,
            2 => 0x1656_67B1_9E37_79F9u64,
            3 => 0x85EB_CA77_C2B2_AE63u64,
            _ => 0x27D4_EB2F_1656_67C5u64,
        };
        let result_tag = if result {
            0xD6E8_FEB8_6659_FD93u64
        } else {
            0xA5A3_58B5_C9CB_4F1Du64
        };

        let mixed = mix64(
            op_tag
                ^ result_tag
                ^ (value as u32 as u64)
                ^ ((size as u64).wrapping_mul(0x1000_0000_01B3)),
        );
        self.event_checksum = mix64(self.event_checksum ^ mixed);
    }
}

#[derive(Clone, Debug)]
pub struct ReferenceList {
    references: Vec<i32>,
    reference_to_index: IntIndexMap,
    linear_search_limit: usize,
}

impl ReferenceList {
    pub fn new(linear_search_limit: usize) -> Self {
        Self {
            references: Vec::new(),
            reference_to_index: IntIndexMap::with_capacity(4),
            linear_search_limit,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.references.len()
    }

    #[inline]
    pub fn as_slice(&self) -> &[i32] {
        &self.references
    }

    #[inline]
    pub fn order_checksum(&self) -> u64 {
        order_checksum(&self.references)
    }

    pub fn clear(&mut self) {
        self.references.clear();
        self.reference_to_index.clear();
    }

    pub fn contains(&self, value: i32) -> bool {
        let count = self.references.len();
        if self.linear_search_limit > 0 && count <= self.linear_search_limit {
            return self.references.iter().any(|&candidate| candidate == value);
        }
        self.reference_to_index.contains_key(value)
    }

    pub fn add(&mut self, value: i32) -> bool {
        assert_valid_value(value);

        let count = self.references.len();
        if self.linear_search_limit > 0 && count <= self.linear_search_limit {
            for &candidate in &self.references {
                if candidate == value {
                    return false;
                }
            }

            self.references.push(value);
            let new_count = count + 1;
            if new_count > self.linear_search_limit {
                self.rebuild_index();
            }
            return true;
        }

        if self.reference_to_index.put_if_absent(value, count).is_some() {
            return false;
        }
        self.references.push(value);
        true
    }

    pub fn remove(&mut self, value: i32) -> bool {
        assert_valid_value(value);

        let count = self.references.len();
        if self.linear_search_limit > 0 && count <= self.linear_search_limit {
            return self.remove_linear(value);
        }
        if self.linear_search_limit > 0
            && self.linear_search_limit <= MAX_LINEAR_TRANSITION_REMOVE_LIMIT
            && count == self.linear_search_limit + 1
        {
            let Some(index) = self.references.iter().position(|&candidate| candidate == value) else {
                return false;
            };
            self.swap_remove_at(index);
            self.reference_to_index.clear();
            return true;
        }

        let Some(index) = self.reference_to_index.remove(value) else {
            return false;
        };
        let end_index = self.references.len() - 1;
        let end = self.references[end_index];
        if index != end_index {
            self.reference_to_index.put(end, index);
        }
        self.references[index] = end;
        self.references.pop();
        if self.linear_search_limit > 0 && end_index <= self.linear_search_limit {
            self.reference_to_index.clear();
        }
        true
    }

    fn remove_linear(&mut self, value: i32) -> bool {
        let Some(index) = self.references.iter().position(|&candidate| candidate == value) else {
            return false;
        };
        self.swap_remove_at(index);
        true
    }

    fn swap_remove_at(&mut self, index: usize) {
        let end_index = self.references.len() - 1;
        let end = self.references[end_index];
        self.references[index] = end;
        self.references.pop();
    }

    fn rebuild_index(&mut self) {
        self.reference_to_index.clear();
        self.reference_to_index.reserve_for(self.references.len());
        for (index, &value) in self.references.iter().enumerate() {
            self.reference_to_index.put(value, index);
        }
    }
}

pub fn run_ops(
    linear_search_limit: usize,
    initial_values: &[i32],
    operations: &[u8],
    values: &[i32],
) -> Result<ReferenceListRunSummary, ReferenceListError> {
    if operations.len() != values.len() {
        return Err(ReferenceListError::LengthMismatch);
    }

    let mut list = ReferenceList::new(linear_search_limit);
    for &value in initial_values {
        if value == EMPTY_KEY {
            return Err(ReferenceListError::InvalidValue);
        }
        list.add(value);
    }

    let mut summary = ReferenceListRunSummary::default();
    for index in 0..operations.len() {
        let op = operations[index];
        let value = values[index];
        if value == EMPTY_KEY {
            return Err(ReferenceListError::InvalidValue);
        }

        let result = match op {
            0 => list.add(value),
            1 => list.remove(value),
            2 => list.contains(value),
            3 => {
                list.clear();
                true
            }
            _ => return Err(ReferenceListError::InvalidOperation),
        };
        summary.record(op, value, result, list.size());
    }

    summary.final_size = list.size() as u64;
    summary.order_checksum = order_checksum(list.as_slice());
    Ok(summary)
}

fn order_checksum(values: &[i32]) -> u64 {
    let mut checksum = 0u64;
    for (index, &value) in values.iter().enumerate() {
        checksum = mix64(
            checksum
                ^ (value as u32 as u64)
                ^ ((index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
        );
    }
    checksum
}

#[inline]
fn assert_valid_value(value: i32) {
    assert!(value != EMPTY_KEY, "i32::MIN is reserved as an empty index key");
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^ (value >> 33)
}

#[derive(Clone, Debug)]
struct IntIndexMap {
    keys: Vec<i32>,
    values: Vec<usize>,
    mask: usize,
    size: usize,
    max_fill: usize,
}

impl IntIndexMap {
    fn with_capacity(expected: usize) -> Self {
        let capacity = table_capacity(expected.max(2));
        Self {
            keys: vec![EMPTY_KEY; capacity],
            values: vec![0; capacity],
            mask: capacity - 1,
            size: 0,
            max_fill: max_fill(capacity),
        }
    }

    fn clear(&mut self) {
        if self.size != 0 {
            self.keys.fill(EMPTY_KEY);
            self.size = 0;
        }
    }

    fn reserve_for(&mut self, expected: usize) {
        if expected + 1 > self.max_fill {
            self.rehash(table_capacity(expected.saturating_mul(2).max(2)));
        }
    }

    #[inline]
    fn contains_key(&self, key: i32) -> bool {
        self.find(key).is_some()
    }

    fn put_if_absent(&mut self, key: i32, value: usize) -> Option<usize> {
        if self.size + 1 > self.max_fill {
            self.rehash(self.keys.len() * 2);
        }

        let mut pos = hash_key(key) & self.mask;
        loop {
            let curr = self.keys[pos];
            if curr == EMPTY_KEY {
                self.keys[pos] = key;
                self.values[pos] = value;
                self.size += 1;
                return None;
            }
            if curr == key {
                return Some(self.values[pos]);
            }
            pos = (pos + 1) & self.mask;
        }
    }

    fn put(&mut self, key: i32, value: usize) -> Option<usize> {
        if self.size + 1 > self.max_fill {
            self.rehash(self.keys.len() * 2);
        }

        let mut pos = hash_key(key) & self.mask;
        loop {
            let curr = self.keys[pos];
            if curr == EMPTY_KEY {
                self.keys[pos] = key;
                self.values[pos] = value;
                self.size += 1;
                return None;
            }
            if curr == key {
                let old = self.values[pos];
                self.values[pos] = value;
                return Some(old);
            }
            pos = (pos + 1) & self.mask;
        }
    }

    fn remove(&mut self, key: i32) -> Option<usize> {
        let pos = self.find(key)?;
        let old = self.values[pos];
        self.size -= 1;
        self.shift_remove_slot(pos);
        Some(old)
    }

    fn find(&self, key: i32) -> Option<usize> {
        let mut pos = hash_key(key) & self.mask;
        loop {
            let curr = self.keys[pos];
            if curr == EMPTY_KEY {
                return None;
            }
            if curr == key {
                return Some(pos);
            }
            pos = (pos + 1) & self.mask;
        }
    }

    fn shift_remove_slot(&mut self, mut pos: usize) {
        loop {
            let last = pos;
            pos = (pos + 1) & self.mask;
            loop {
                let curr = self.keys[pos];
                if curr == EMPTY_KEY {
                    self.keys[last] = EMPTY_KEY;
                    return;
                }

                let slot = hash_key(curr) & self.mask;
                let must_shift = if last <= pos {
                    last >= slot || slot > pos
                } else {
                    last >= slot && slot > pos
                };
                if must_shift {
                    break;
                }
                pos = (pos + 1) & self.mask;
            }

            self.keys[last] = self.keys[pos];
            self.values[last] = self.values[pos];
        }
    }

    fn rehash(&mut self, capacity: usize) {
        let new_capacity = table_capacity(capacity);
        let old_keys = std::mem::replace(&mut self.keys, vec![EMPTY_KEY; new_capacity]);
        let old_values = std::mem::replace(&mut self.values, vec![0; new_capacity]);
        self.mask = new_capacity - 1;
        self.max_fill = max_fill(new_capacity);
        self.size = 0;

        for (index, key) in old_keys.into_iter().enumerate() {
            if key != EMPTY_KEY {
                self.put(key, old_values[index]);
            }
        }
    }
}

#[inline]
fn hash_key(key: i32) -> usize {
    let mut value = key as u32;
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;
    value as usize
}

fn table_capacity(expected: usize) -> usize {
    expected.next_power_of_two().max(4)
}

fn max_fill(capacity: usize) -> usize {
    ((capacity * 3) / 4).max(1).min(capacity - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_linear_add_remove_contains() {
        let mut list = ReferenceList::new(2);
        assert!(list.add(10));
        assert!(list.add(11));
        assert!(!list.add(10));
        assert!(list.contains(11));
        assert!(list.remove(10));
        assert_eq!(list.as_slice(), &[11]);
        assert!(!list.contains(10));
    }

    #[test]
    fn crossing_linear_limit_builds_index() {
        let mut list = ReferenceList::new(2);
        assert!(list.add(1));
        assert!(list.add(2));
        assert!(list.add(3));
        assert!(list.reference_to_index.contains_key(1));
        assert!(list.reference_to_index.contains_key(2));
        assert!(list.reference_to_index.contains_key(3));
        assert!(!list.add(2));
    }

    #[test]
    fn transition_remove_clears_index_and_preserves_swap_order() {
        let mut list = ReferenceList::new(2);
        list.add(1);
        list.add(2);
        list.add(3);
        assert!(list.remove(2));
        assert_eq!(list.as_slice(), &[1, 3]);
        assert_eq!(list.reference_to_index.size, 0);
        assert!(list.contains(3));
    }

    #[test]
    fn dense_remove_updates_moved_index() {
        let mut list = ReferenceList::new(2);
        for value in 1..=6 {
            assert!(list.add(value));
        }

        assert!(list.remove(2));
        assert_eq!(list.as_slice(), &[1, 6, 3, 4, 5]);
        assert!(list.contains(6));
        assert!(!list.add(6));
        assert!(list.remove(6));
        assert_eq!(list.as_slice(), &[1, 5, 3, 4]);
    }

    #[test]
    fn run_ops_summary_is_stable() {
        let ops = [0, 0, 0, 2, 1, 0, 1, 3, 0, 2];
        let values = [1, 2, 3, 2, 1, 4, 3, 0, 9, 9];
        let summary = run_ops(2, &[], &ops, &values).unwrap();
        assert_eq!(summary.add_true, 5);
        assert_eq!(summary.remove_true, 2);
        assert_eq!(summary.contains_true, 2);
        assert_eq!(summary.false_count, 0);
        assert_eq!(summary.final_size, 1);
        assert_ne!(summary.event_checksum, 0);
        assert_ne!(summary.order_checksum, 0);
    }

    #[test]
    fn rejects_bad_ops_and_reserved_value() {
        assert_eq!(
            run_ops(2, &[], &[4], &[1]),
            Err(ReferenceListError::InvalidOperation)
        );
        assert_eq!(
            run_ops(2, &[EMPTY_KEY], &[], &[]),
            Err(ReferenceListError::InvalidValue)
        );
        assert_eq!(
            run_ops(2, &[], &[0], &[]),
            Err(ReferenceListError::LengthMismatch)
        );
    }

    #[test]
    fn random_ops_match_vec_model_with_linear_mode() {
        assert_random_ops_match(2, 0x5EED_1234_89AB_CDEF);
    }

    #[test]
    fn random_ops_match_vec_model_with_hash_mode() {
        assert_random_ops_match(0, 0xA17C_5EED_2468_1357);
    }

    fn assert_random_ops_match(linear_search_limit: usize, mut state: u64) {
        let mut list = ReferenceList::new(linear_search_limit);
        let mut model = Vec::new();

        for _ in 0..50_000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let op = ((state >> 32) as u8) & 3;
            let value = ((state >> 16) as i32) & 63;

            match op {
                0 => {
                    let expected = !model.iter().any(|&candidate| candidate == value);
                    let actual = list.add(value);
                    if actual != expected {
                        panic!("add mismatch value={value} expected={expected} actual={actual}");
                    }
                    if actual {
                        model.push(value);
                    }
                }
                1 => {
                    let expected_index = model.iter().position(|&candidate| candidate == value);
                    let actual = list.remove(value);
                    if actual != expected_index.is_some() {
                        panic!("remove mismatch value={value} expected={:?} actual={actual}", expected_index);
                    }
                    if let Some(index) = expected_index {
                        model.swap_remove(index);
                    }
                }
                2 => {
                    let actual = list.contains(value);
                    let expected = model.iter().any(|&candidate| candidate == value);
                    if actual != expected {
                        panic!("contains mismatch value={value} expected={expected} actual={actual}");
                    }
                }
                3 => {
                    list.clear();
                    model.clear();
                }
                _ => unreachable!(),
            };

            assert_eq!(list.as_slice(), model.as_slice());
        }
    }
}
