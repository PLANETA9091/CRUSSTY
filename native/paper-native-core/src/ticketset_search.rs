use std::cmp::Ordering;
use std::sync::OnceLock;

pub const SUMMARY_FIELDS: usize = 1;
const SET_COUNT: usize = 4096;
const OP_COUNT: usize = 1 << 18;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TicketSetSearchError {
    InvalidIterations,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TicketSetSearchSummary {
    pub value: i64,
}

#[derive(Clone, Copy)]
struct TypeInfo {
    id: i64,
    comparator: bool,
}

#[derive(Clone, Copy)]
struct Ticket {
    type_index: usize,
    level: i32,
    identifier: i32,
}

#[derive(Clone, Copy)]
struct Operation {
    mode: u8,
    set_index: usize,
    key: Ticket,
}

struct SearchData {
    base_tickets: Vec<Vec<Ticket>>,
    operations: Vec<Operation>,
}

static SEARCH_DATA: OnceLock<SearchData> = OnceLock::new();

const TYPES: [TypeInfo; 6] = [
    TypeInfo {
        id: 0,
        comparator: false,
    },
    TypeInfo {
        id: 1,
        comparator: false,
    },
    TypeInfo {
        id: 2,
        comparator: true,
    },
    TypeInfo {
        id: 3,
        comparator: false,
    },
    TypeInfo {
        id: 4,
        comparator: true,
    },
    TypeInfo {
        id: 5,
        comparator: false,
    },
];

#[inline]
pub fn binary_summary(iterations: usize) -> Result<TicketSetSearchSummary, TicketSetSearchError> {
    run_summary(iterations, RunKind::Binary, 0)
}

#[inline]
pub fn unchecked_binary_summary(
    iterations: usize,
) -> Result<TicketSetSearchSummary, TicketSetSearchError> {
    run_summary(iterations, RunKind::UncheckedBinary, 0)
}

#[inline]
pub fn linear_summary(
    iterations: usize,
    linear_limit: usize,
) -> Result<TicketSetSearchSummary, TicketSetSearchError> {
    run_summary(iterations, RunKind::Linear, linear_limit)
}

#[derive(Clone, Copy)]
enum RunKind {
    Binary,
    UncheckedBinary,
    Linear,
}

fn run_summary(
    iterations: usize,
    kind: RunKind,
    linear_limit: usize,
) -> Result<TicketSetSearchSummary, TicketSetSearchError> {
    if iterations == 0 {
        return Err(TicketSetSearchError::InvalidIterations);
    }

    let data = data();
    let mut value = 0i64;

    match kind {
        RunKind::Binary => {
            let mut sets = create_binary_sets(data);
            for i in 0..iterations {
                let op = data.operations[i & (OP_COUNT - 1)];
                let set = &mut sets[op.set_index];
                value = value.wrapping_add(apply_binary(set, op));
            }
            value = value.wrapping_add(hash_binary(&sets));
        }
        RunKind::UncheckedBinary => {
            let mut sets = create_unchecked_binary_sets(data);
            for i in 0..iterations {
                let op = data.operations[i & (OP_COUNT - 1)];
                let set = &mut sets[op.set_index];
                value = value.wrapping_add(apply_unchecked_binary(set, op));
            }
            value = value.wrapping_add(hash_unchecked_binary(&sets));
        }
        RunKind::Linear => {
            let mut sets = create_linear_sets(data, linear_limit);
            for i in 0..iterations {
                let op = data.operations[i & (OP_COUNT - 1)];
                let set = &mut sets[op.set_index];
                value = value.wrapping_add(apply_linear(set, op));
            }
            value = value.wrapping_add(hash_linear(&sets));
        }
    }

    Ok(TicketSetSearchSummary { value })
}

fn data() -> &'static SearchData {
    SEARCH_DATA.get_or_init(build_data)
}

fn build_data() -> SearchData {
    let mut base_tickets = Vec::with_capacity(SET_COUNT);
    for set in 0..SET_COUNT {
        let size = match set & 7 {
            0 | 1 | 2 => 1 + (set & 3),
            3 | 4 | 5 => 5 + (set & 7),
            _ => 12 + (set & 3),
        };
        let mut builder = TicketSetBinary::new(size);
        for index in 0..size {
            builder.add(make_ticket(set, index));
        }
        base_tickets.push(builder.copy_tickets());
    }

    let mut random = JavaRandom::new(0x71c4e7);
    let mut operations = Vec::with_capacity(OP_COUNT);
    for i in 0..OP_COUNT {
        let set_index = random.next_int(SET_COUNT as i32) as usize;
        let tickets = &base_tickets[set_index];
        let mode = (i & 3) as u8;
        let key = if mode == 2 || mode == 3 {
            make_missing_ticket(set_index, random.next_int(64))
        } else {
            copy_ticket(&tickets[random.next_int(tickets.len() as i32) as usize])
        };
        operations.push(Operation {
            mode,
            set_index,
            key,
        });
    }

    SearchData {
        base_tickets,
        operations,
    }
}

fn create_binary_sets(data: &SearchData) -> Vec<TicketSetBinary> {
    let mut sets = Vec::with_capacity(SET_COUNT);
    for base in &data.base_tickets {
        let mut set = TicketSetBinary::new(base.len() + 4);
        for &ticket in base {
            set.add(ticket);
        }
        sets.push(set);
    }
    sets
}

fn create_unchecked_binary_sets(data: &SearchData) -> Vec<TicketSetUncheckedBinary> {
    let mut sets = Vec::with_capacity(SET_COUNT);
    for base in &data.base_tickets {
        let mut set = TicketSetUncheckedBinary::new(base.len() + 4);
        for &ticket in base {
            set.add(ticket);
        }
        sets.push(set);
    }
    sets
}

fn create_linear_sets(data: &SearchData, linear_limit: usize) -> Vec<TicketSetLinear> {
    let mut sets = Vec::with_capacity(SET_COUNT);
    for base in &data.base_tickets {
        let mut set = TicketSetLinear::new(base.len() + 4, linear_limit);
        for &ticket in base {
            set.add(ticket);
        }
        sets.push(set);
    }
    sets
}

fn apply_binary(set: &mut TicketSetBinary, op: Operation) -> i64 {
    match op.mode {
        0 => set.replace(copy_ticket(&op.key)).sort_value(),
        1 => {
            let removed = set.remove_and_get(&op.key).expect("missing existing key");
            set.add(removed);
            removed.sort_value()
        }
        2 => {
            if set.remove_and_get(&op.key).is_none() {
                7
            } else {
                -99_999
            }
        }
        _ => {
            let added = copy_ticket(&op.key);
            assert!(set.add(added), "expected insert");
            assert!(set.remove(&added), "expected remove");
            added.sort_value()
        }
    }
}

fn apply_unchecked_binary(set: &mut TicketSetUncheckedBinary, op: Operation) -> i64 {
    match op.mode {
        0 => set.replace(copy_ticket(&op.key)).sort_value(),
        1 => {
            let removed = set.remove_and_get(&op.key).expect("missing existing key");
            set.add(removed);
            removed.sort_value()
        }
        2 => {
            if set.remove_and_get(&op.key).is_none() {
                7
            } else {
                -99_999
            }
        }
        _ => {
            let added = copy_ticket(&op.key);
            assert!(set.add(added), "expected insert");
            assert!(set.remove(&added), "expected remove");
            added.sort_value()
        }
    }
}

fn apply_linear(set: &mut TicketSetLinear, op: Operation) -> i64 {
    match op.mode {
        0 => set.replace(copy_ticket(&op.key)).sort_value(),
        1 => {
            let removed = set.remove_and_get(&op.key).expect("missing existing key");
            set.add(removed);
            removed.sort_value()
        }
        2 => {
            if set.remove_and_get(&op.key).is_none() {
                7
            } else {
                -99_999
            }
        }
        _ => {
            let added = copy_ticket(&op.key);
            assert!(set.add(added), "expected insert");
            assert!(set.remove(&added), "expected remove");
            added.sort_value()
        }
    }
}

fn hash_binary(sets: &[TicketSetBinary]) -> i64 {
    let mut hash = 0i64;
    for set in sets {
        hash = hash.wrapping_mul(31).wrapping_add(set.hash());
    }
    hash
}

fn hash_unchecked_binary(sets: &[TicketSetUncheckedBinary]) -> i64 {
    let mut hash = 0i64;
    for set in sets {
        hash = hash.wrapping_mul(31).wrapping_add(set.hash());
    }
    hash
}

fn hash_linear(sets: &[TicketSetLinear]) -> i64 {
    let mut hash = 0i64;
    for set in sets {
        hash = hash.wrapping_mul(31).wrapping_add(set.hash());
    }
    hash
}

fn make_ticket(set: usize, index: usize) -> Ticket {
    let type_index = (set + index * 3) % TYPES.len();
    let level = 20 + (((set * 5 + index * 7) & 31) as i32);
    let identifier = if TYPES[type_index].comparator {
        ((set * 131) ^ (index * 17)) as i32
    } else {
        0
    };
    Ticket {
        type_index,
        level,
        identifier,
    }
}

fn make_missing_ticket(set: usize, salt: i32) -> Ticket {
    let type_index = (set + (salt as usize) * 5 + 1) % TYPES.len();
    let level = 80 + (salt & 31);
    let identifier = if TYPES[type_index].comparator {
        ((set * 193) ^ ((salt as usize) * 29) ^ 0x55aa) as i32
    } else {
        0
    };
    Ticket {
        type_index,
        level,
        identifier,
    }
}

fn copy_ticket(ticket: &Ticket) -> Ticket {
    *ticket
}

fn compare_ticket(a: &Ticket, b: &Ticket) -> Ordering {
    match a.level.cmp(&b.level) {
        Ordering::Equal => {}
        non_eq => return non_eq,
    }

    match TYPES[a.type_index].id.cmp(&TYPES[b.type_index].id) {
        Ordering::Equal => {}
        non_eq => return non_eq,
    }

    if TYPES[a.type_index].comparator {
        a.identifier.cmp(&b.identifier)
    } else {
        Ordering::Equal
    }
}

fn binary_search(tickets: &[Ticket], key: &Ticket) -> Result<usize, usize> {
    let mut start = 0usize;
    let mut end = tickets.len();
    while start < end {
        let middle = (start + end) >> 1;
        match compare_ticket(&tickets[middle], key) {
            Ordering::Less => start = middle + 1,
            Ordering::Greater => end = middle,
            Ordering::Equal => return Ok(middle),
        }
    }
    Err(start)
}

fn binary_search_unchecked(tickets: &[Ticket], key: &Ticket) -> Result<usize, usize> {
    let mut start = 0usize;
    let mut end = tickets.len();
    while start < end {
        let middle = (start + end) >> 1;
        match compare_ticket(&tickets[middle], key) {
            Ordering::Less => start = middle + 1,
            Ordering::Greater => end = middle,
            Ordering::Equal => return Ok(middle),
        }
    }
    Err(start)
}

fn linear_or_binary_search(tickets: &[Ticket], key: &Ticket, linear_limit: usize) -> Result<usize, usize> {
    if tickets.len() <= linear_limit {
        for (index, ticket) in tickets.iter().enumerate() {
            match compare_ticket(ticket, key) {
                Ordering::Less => {}
                Ordering::Equal => return Ok(index),
                Ordering::Greater => return Err(index),
            }
        }
        Err(tickets.len())
    } else {
        binary_search(tickets, key)
    }
}

struct TicketSetBinary {
    tickets: Vec<Ticket>,
}

impl TicketSetBinary {
    fn new(initial_capacity: usize) -> Self {
        Self {
            tickets: Vec::with_capacity(initial_capacity),
        }
    }

    fn replace(&mut self, ticket: Ticket) -> Ticket {
        match binary_search(&self.tickets, &ticket) {
            Ok(index) => std::mem::replace(&mut self.tickets[index], ticket),
            Err(index) => {
                self.add_at(index, ticket);
                ticket
            }
        }
    }

    fn add(&mut self, ticket: Ticket) -> bool {
        match binary_search(&self.tickets, &ticket) {
            Ok(_) => false,
            Err(index) => {
                self.add_at(index, ticket);
                true
            }
        }
    }

    fn remove(&mut self, ticket: &Ticket) -> bool {
        match binary_search(&self.tickets, ticket) {
            Ok(index) => {
                self.remove_at(index);
                true
            }
            Err(_) => false,
        }
    }

    fn remove_and_get(&mut self, ticket: &Ticket) -> Option<Ticket> {
        match binary_search(&self.tickets, ticket) {
            Ok(index) => Some(self.remove_at(index)),
            Err(_) => None,
        }
    }

    fn add_at(&mut self, index: usize, ticket: Ticket) {
        if index == self.tickets.len() {
            self.tickets.push(ticket);
        } else {
            self.tickets.insert(index, ticket);
        }
    }

    fn remove_at(&mut self, index: usize) -> Ticket {
        self.tickets.remove(index)
    }

    fn copy_tickets(&self) -> Vec<Ticket> {
        self.tickets.clone()
    }

    fn hash(&self) -> i64 {
        let mut hash = self.tickets.len() as i64;
        for ticket in &self.tickets {
            hash = hash.wrapping_mul(31).wrapping_add(ticket.sort_value());
        }
        hash
    }
}

struct TicketSetUncheckedBinary {
    tickets: Vec<Ticket>,
}

impl TicketSetUncheckedBinary {
    fn new(initial_capacity: usize) -> Self {
        Self {
            tickets: Vec::with_capacity(initial_capacity),
        }
    }

    fn replace(&mut self, ticket: Ticket) -> Ticket {
        match binary_search_unchecked(&self.tickets, &ticket) {
            Ok(index) => std::mem::replace(&mut self.tickets[index], ticket),
            Err(index) => {
                self.add_at(index, ticket);
                ticket
            }
        }
    }

    fn add(&mut self, ticket: Ticket) -> bool {
        match binary_search_unchecked(&self.tickets, &ticket) {
            Ok(_) => false,
            Err(index) => {
                self.add_at(index, ticket);
                true
            }
        }
    }

    fn remove(&mut self, ticket: &Ticket) -> bool {
        match binary_search_unchecked(&self.tickets, ticket) {
            Ok(index) => {
                self.remove_at(index);
                true
            }
            Err(_) => false,
        }
    }

    fn remove_and_get(&mut self, ticket: &Ticket) -> Option<Ticket> {
        match binary_search_unchecked(&self.tickets, ticket) {
            Ok(index) => Some(self.remove_at(index)),
            Err(_) => None,
        }
    }

    fn add_at(&mut self, index: usize, ticket: Ticket) {
        if index == self.tickets.len() {
            self.tickets.push(ticket);
        } else {
            self.tickets.insert(index, ticket);
        }
    }

    fn remove_at(&mut self, index: usize) -> Ticket {
        self.tickets.remove(index)
    }

    fn hash(&self) -> i64 {
        let mut hash = self.tickets.len() as i64;
        for ticket in &self.tickets {
            hash = hash.wrapping_mul(31).wrapping_add(ticket.sort_value());
        }
        hash
    }
}

struct TicketSetLinear {
    tickets: Vec<Ticket>,
    linear_limit: usize,
}

impl TicketSetLinear {
    fn new(initial_capacity: usize, linear_limit: usize) -> Self {
        Self {
            tickets: Vec::with_capacity(initial_capacity),
            linear_limit,
        }
    }

    fn search(&self, key: &Ticket) -> Result<usize, usize> {
        linear_or_binary_search(&self.tickets, key, self.linear_limit)
    }

    fn replace(&mut self, ticket: Ticket) -> Ticket {
        match self.search(&ticket) {
            Ok(index) => std::mem::replace(&mut self.tickets[index], ticket),
            Err(index) => {
                self.add_at(index, ticket);
                ticket
            }
        }
    }

    fn add(&mut self, ticket: Ticket) -> bool {
        match self.search(&ticket) {
            Ok(_) => false,
            Err(index) => {
                self.add_at(index, ticket);
                true
            }
        }
    }

    fn remove(&mut self, ticket: &Ticket) -> bool {
        match self.search(ticket) {
            Ok(index) => {
                self.remove_at(index);
                true
            }
            Err(_) => false,
        }
    }

    fn remove_and_get(&mut self, ticket: &Ticket) -> Option<Ticket> {
        match self.search(ticket) {
            Ok(index) => Some(self.remove_at(index)),
            Err(_) => None,
        }
    }

    fn add_at(&mut self, index: usize, ticket: Ticket) {
        if index == self.tickets.len() {
            self.tickets.push(ticket);
        } else {
            self.tickets.insert(index, ticket);
        }
    }

    fn remove_at(&mut self, index: usize) -> Ticket {
        self.tickets.remove(index)
    }

    fn hash(&self) -> i64 {
        let mut hash = self.tickets.len() as i64;
        for ticket in &self.tickets {
            hash = hash.wrapping_mul(31).wrapping_add(ticket.sort_value());
        }
        hash
    }
}

impl Ticket {
    fn sort_value(&self) -> i64 {
        ((self.level as i64) << 32)
            ^ (TYPES[self.type_index].id << 16)
            ^ (self.identifier as i64)
    }
}

impl JavaRandom {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1u64 << 48) - 1;

    fn new(seed: u64) -> Self {
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if (bound & -bound) == bound {
            return (((bound as i64) * (self.next(31) as i64)) >> 31) as i32;
        }

        loop {
            let bits = self.next(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }
}

struct JavaRandom {
    seed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_and_linear_match_on_small_run() {
        let binary = binary_summary(10_000).unwrap();
        let unchecked = unchecked_binary_summary(10_000).unwrap();
        let linear = linear_summary(10_000, 4).unwrap();
        assert_eq!(binary, unchecked);
        assert_eq!(binary, linear);
    }

    #[test]
    fn rejects_zero_iterations() {
        assert_eq!(
            binary_summary(0),
            Err(TicketSetSearchError::InvalidIterations)
        );
    }
}
