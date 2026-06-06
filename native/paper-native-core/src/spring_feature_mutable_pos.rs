pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const HASH_A: u64 = 0x9E37_79B9_7F4A_7C15;
const HASH_B: u64 = 0xC2B2_AE3D_27D4_EB4F;
const HASH_C: u64 = 0x1656_67B1_9E37_79F9;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SpringFeatureMutablePosSummary {
    pub count: u64,
    pub success_count: u64,
    pub checksum: u64,
    pub last_decision: u64,
}

pub fn old_batch_summary(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    requires_below: &[bool],
    rock_count: &[i32],
    hole_count: &[i32],
    iterations: usize,
) -> SpringFeatureMutablePosSummary {
    run_batch_summary(
        xs,
        ys,
        zs,
        requires_below,
        rock_count,
        hole_count,
        iterations,
        old_place,
    )
}

pub fn mutable_batch_summary(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    requires_below: &[bool],
    rock_count: &[i32],
    hole_count: &[i32],
    iterations: usize,
) -> SpringFeatureMutablePosSummary {
    run_batch_summary(
        xs,
        ys,
        zs,
        requires_below,
        rock_count,
        hole_count,
        iterations,
        mutable_place,
    )
}

fn run_batch_summary<F>(
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    requires_below: &[bool],
    rock_count: &[i32],
    hole_count: &[i32],
    iterations: usize,
    mut place: F,
) -> SpringFeatureMutablePosSummary
where
    F: FnMut(usize, &[i32], &[i32], &[i32], &[bool], &[i32], &[i32]) -> bool,
{
    let positions = xs.len();
    debug_assert_eq!(positions, ys.len());
    debug_assert_eq!(positions, zs.len());
    debug_assert_eq!(positions, requires_below.len());
    debug_assert_eq!(positions, rock_count.len());
    debug_assert_eq!(positions, hole_count.len());

    if iterations == 0 {
        return SpringFeatureMutablePosSummary::default();
    }

    debug_assert!(positions > 0 && positions.is_power_of_two());

    let mask = positions - 1;
    let mut success_count = 0u64;
    let mut checksum = 0u64;
    let mut last_decision = 0u64;

    for i in 0..iterations {
        let index = ((i as u32).wrapping_mul(17) as usize) & mask;
        let placed = place(index, xs, ys, zs, requires_below, rock_count, hole_count);
        let decision = if placed { 1u64 } else { 0u64 };
        success_count += decision;
        last_decision = decision;
        if placed {
            checksum = mix(checksum, index as i32);
        }
        checksum = mix64(
            checksum
                ^ decision
                ^ ((i as u64).wrapping_mul(MIX_GAMMA))
                ^ ((iterations as u64).rotate_left(13)),
        );
    }

    SpringFeatureMutablePosSummary {
        count: iterations as u64,
        success_count,
        checksum,
        last_decision,
    }
}

fn old_place(
    index: usize,
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    requires_below: &[bool],
    rock_count: &[i32],
    hole_count: &[i32],
) -> bool {
    let pos = Position::new(xs[index], ys[index], zs[index]);
    if !is_valid(pos.above()) {
        return false;
    }
    if requires_below[index] && !is_valid(pos.below()) {
        return false;
    }
    if !is_air(pos) && !is_valid(pos) {
        return false;
    }

    let mut rock = 0;
    if is_valid(pos.west()) {
        rock += 1;
    }
    if is_valid(pos.east()) {
        rock += 1;
    }
    if is_valid(pos.north()) {
        rock += 1;
    }
    if is_valid(pos.south()) {
        rock += 1;
    }
    if is_valid(pos.below()) {
        rock += 1;
    }

    let mut holes = 0;
    if is_air(pos.west()) {
        holes += 1;
    }
    if is_air(pos.east()) {
        holes += 1;
    }
    if is_air(pos.north()) {
        holes += 1;
    }
    if is_air(pos.south()) {
        holes += 1;
    }
    if is_air(pos.below()) {
        holes += 1;
    }

    rock == rock_count[index] && holes == hole_count[index]
}

fn mutable_place(
    index: usize,
    xs: &[i32],
    ys: &[i32],
    zs: &[i32],
    requires_below: &[bool],
    rock_count: &[i32],
    hole_count: &[i32],
) -> bool {
    let mut pos = MutablePosition::default();
    let x = xs[index];
    let y = ys[index];
    let z = zs[index];

    if !is_valid(pos.set(x, y.wrapping_add(1), z)) {
        return false;
    }
    if requires_below[index] && !is_valid(pos.set(x, y.wrapping_sub(1), z)) {
        return false;
    }
    if !is_air(pos.set(x, y, z)) && !is_valid(pos.as_position()) {
        return false;
    }

    let mut rock = 0;
    if is_valid(pos.set(x.wrapping_sub(1), y, z)) {
        rock += 1;
    }
    if is_valid(pos.set(x.wrapping_add(1), y, z)) {
        rock += 1;
    }
    if is_valid(pos.set(x, y, z.wrapping_sub(1))) {
        rock += 1;
    }
    if is_valid(pos.set(x, y, z.wrapping_add(1))) {
        rock += 1;
    }
    if is_valid(pos.set(x, y.wrapping_sub(1), z)) {
        rock += 1;
    }

    let mut holes = 0;
    if is_air(pos.set(x.wrapping_sub(1), y, z)) {
        holes += 1;
    }
    if is_air(pos.set(x.wrapping_add(1), y, z)) {
        holes += 1;
    }
    if is_air(pos.set(x, y, z.wrapping_sub(1))) {
        holes += 1;
    }
    if is_air(pos.set(x, y, z.wrapping_add(1))) {
        holes += 1;
    }
    if is_air(pos.set(x, y.wrapping_sub(1), z)) {
        holes += 1;
    }

    rock == rock_count[index] && holes == hole_count[index]
}

fn is_valid(pos: Position) -> bool {
    (hash(pos.x, pos.y, pos.z) & 7) != 0
}

fn is_air(pos: Position) -> bool {
    (hash(pos.x, pos.y, pos.z).rotate_left(17) & 3) == 0
}

fn hash(x: i32, y: i32, z: i32) -> u64 {
    let mut value = (x as u32 as u64).wrapping_mul(HASH_A);
    value ^= (y as u32 as u64).wrapping_mul(HASH_B);
    value = value.rotate_left(31) ^ (z as u32 as u64).wrapping_mul(HASH_C);
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value
}

#[inline]
fn mix(checksum: u64, value: i32) -> u64 {
    (checksum.wrapping_mul(MIX_GAMMA)) ^ (value as u32 as u64)
}

#[inline]
fn mix64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^ (value >> 33)
}

#[derive(Clone, Copy, Debug, Default)]
struct Position {
    x: i32,
    y: i32,
    z: i32,
}

impl Position {
    #[inline]
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    fn above(self) -> Self {
        Self {
            y: self.y.wrapping_add(1),
            ..self
        }
    }

    #[inline]
    fn below(self) -> Self {
        Self {
            y: self.y.wrapping_sub(1),
            ..self
        }
    }

    #[inline]
    fn west(self) -> Self {
        Self {
            x: self.x.wrapping_sub(1),
            ..self
        }
    }

    #[inline]
    fn east(self) -> Self {
        Self {
            x: self.x.wrapping_add(1),
            ..self
        }
    }

    #[inline]
    fn north(self) -> Self {
        Self {
            z: self.z.wrapping_sub(1),
            ..self
        }
    }

    #[inline]
    fn south(self) -> Self {
        Self {
            z: self.z.wrapping_add(1),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct MutablePosition {
    x: i32,
    y: i32,
    z: i32,
}

impl MutablePosition {
    #[inline]
    fn set(&mut self, x: i32, y: i32, z: i32) -> Position {
        self.x = x;
        self.y = y;
        self.z = z;
        self.as_position()
    }

    #[inline]
    fn as_position(self) -> Position {
        Position::new(self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_mutable_match_on_regular_inputs() {
        let xs = [0, 1, -2, 30, -30, 128, -128, 7];
        let ys = [0, -1, 2, 63, -64, 12, 34, 56];
        let zs = [0, 3, -4, 11, -22, 99, -100, 5];
        let requires_below = [false, true, false, true, false, true, false, true];
        let rock_count = [3, 4, 3, 4, 3, 4, 3, 4];
        let hole_count = [1, 2, 1, 2, 1, 2, 1, 2];

        let old = old_batch_summary(&xs, &ys, &zs, &requires_below, &rock_count, &hole_count, 64);
        let mutable = mutable_batch_summary(&xs, &ys, &zs, &requires_below, &rock_count, &hole_count, 64);

        assert_eq!(old, mutable);
        assert_eq!(old.count, 64);
    }

    #[test]
    fn zero_iterations_are_empty() {
        let empty_i32: [i32; 0] = [];
        let empty_bool: [bool; 0] = [];
        let summary = old_batch_summary(&empty_i32, &empty_i32, &empty_i32, &empty_bool, &empty_i32, &empty_i32, 0);
        assert_eq!(summary, SpringFeatureMutablePosSummary::default());
    }

    #[test]
    fn repeated_runs_are_stable() {
        let xs = [10, -10, 20, -20];
        let ys = [1, 2, 3, 4];
        let zs = [5, 6, 7, 8];
        let requires_below = [true, false, true, false];
        let rock_count = [3, 4, 3, 4];
        let hole_count = [1, 2, 1, 2];

        let first = mutable_batch_summary(&xs, &ys, &zs, &requires_below, &rock_count, &hole_count, 17);
        let second = mutable_batch_summary(&xs, &ys, &zs, &requires_below, &rock_count, &hole_count, 17);

        assert_eq!(first, second);
        assert_eq!(first, old_batch_summary(&xs, &ys, &zs, &requires_below, &rock_count, &hole_count, 17));
    }
}
