pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const HEIGHTMAP_TAG: u64 = 0x1EAD_C00A_AA55_9911;
const COLUMN_COUNT: usize = 256;
const MIN_Y: i32 = -64;
const HEIGHT: usize = 192;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LevelChunkHeightmapSummary {
    pub count: u64,
    pub total: u64,
    pub checksum: u64,
    pub last_total: u64,
}

pub fn old_four_update_summary(batches: usize) -> LevelChunkHeightmapSummary {
    run_summary(batches, Mode::Old)
}

pub fn new_combined_update_summary(batches: usize) -> LevelChunkHeightmapSummary {
    run_summary(batches, Mode::New)
}

#[derive(Clone, Copy)]
enum Mode {
    Old,
    New,
}

fn run_summary(batches: usize, mode: Mode) -> LevelChunkHeightmapSummary {
    if batches == 0 {
        return LevelChunkHeightmapSummary::default();
    }

    let shape_digest = mix64(
        HEIGHTMAP_TAG
            ^ match mode {
                Mode::Old => 0x10,
                Mode::New => 0x20,
            }
            ^ ((batches as u64) << 8),
    );
    let chunk = Chunk::new();
    let value = run_once(batches, &chunk, mode);
    let mut checksum = 0u64;
    for iteration in 0..batches {
        checksum = mix64(
            checksum
                ^ value
                ^ shape_digest
                ^ ((iteration as u64).wrapping_mul(MIX_GAMMA))
                ^ ((COLUMN_COUNT as u64) << 11),
        );
    }

    LevelChunkHeightmapSummary {
        count: batches as u64,
        total: value.wrapping_mul(batches as u64),
        checksum,
        last_total: value,
    }
}

fn run_once(batches: usize, chunk: &Chunk, mode: Mode) -> u64 {
    let mut motion_blocking = Heightmap::new(Type::MotionBlocking);
    let mut no_leaves = Heightmap::new(Type::MotionBlockingNoLeaves);
    let mut ocean_floor = Heightmap::new(Type::OceanFloor);
    let mut world_surface = Heightmap::new(Type::WorldSurface);
    let mut checksum = 0u64;

    for batch in 0..batches {
        reset(&mut motion_blocking, &mut no_leaves, &mut ocean_floor, &mut world_surface);

        for column in 0..COLUMN_COUNT {
            let x = (column & 15) as i32;
            let z = (column >> 4) as i32;
            match mode {
                Mode::Old => {
                    update_old(chunk, &mut motion_blocking, &mut no_leaves, &mut ocean_floor, &mut world_surface, x, 79, z, State::Air);
                    update_old(chunk, &mut motion_blocking, &mut no_leaves, &mut ocean_floor, &mut world_surface, x, 80, z, State::Stone);
                    update_old(chunk, &mut motion_blocking, &mut no_leaves, &mut ocean_floor, &mut world_surface, x, 20, z, State::Stone);
                    update_old(chunk, &mut motion_blocking, &mut no_leaves, &mut ocean_floor, &mut world_surface, x, 80, z, State::Air);
                }
                Mode::New => {
                    update_combined(&mut motion_blocking, &mut no_leaves, &mut ocean_floor, &mut world_surface, chunk, x, 79, z, State::Air);
                    update_combined(&mut motion_blocking, &mut no_leaves, &mut ocean_floor, &mut world_surface, chunk, x, 80, z, State::Stone);
                    update_combined(&mut motion_blocking, &mut no_leaves, &mut ocean_floor, &mut world_surface, chunk, x, 20, z, State::Stone);
                    update_combined(&mut motion_blocking, &mut no_leaves, &mut ocean_floor, &mut world_surface, chunk, x, 80, z, State::Air);
                }
            }
        }

        checksum ^= sum(&motion_blocking, &no_leaves, &ocean_floor, &world_surface) + batch as u64;
    }

    checksum
}

fn reset(heightmap0: &mut Heightmap, heightmap1: &mut Heightmap, heightmap2: &mut Heightmap, heightmap3: &mut Heightmap) {
    heightmap0.first_available.fill(80);
    heightmap1.first_available.fill(80);
    heightmap2.first_available.fill(80);
    heightmap3.first_available.fill(80);
}

fn sum(heightmap0: &Heightmap, heightmap1: &Heightmap, heightmap2: &Heightmap, heightmap3: &Heightmap) -> u64 {
    heightmap0
        .first_available
        .iter()
        .chain(heightmap1.first_available.iter())
        .chain(heightmap2.first_available.iter())
        .chain(heightmap3.first_available.iter())
        .fold(0u64, |sum, value| sum.wrapping_add(*value as u64))
}

fn update_old(
    chunk: &Chunk,
    motion_blocking: &mut Heightmap,
    no_leaves: &mut Heightmap,
    ocean_floor: &mut Heightmap,
    world_surface: &mut Heightmap,
    x: i32,
    y: i32,
    z: i32,
    state: State,
) {
    motion_blocking.update(chunk, x, y, z, state);
    no_leaves.update(chunk, x, y, z, state);
    ocean_floor.update(chunk, x, y, z, state);
    world_surface.update(chunk, x, y, z, state);
}

fn update_combined(
    heightmap0: &mut Heightmap,
    heightmap1: &mut Heightmap,
    heightmap2: &mut Heightmap,
    heightmap3: &mut Heightmap,
    chunk: &Chunk,
    x: i32,
    y: i32,
    z: i32,
    state: State,
) {
    let mut h0 = Some((heightmap0.ty, heightmap0.get_first_available(x, z)));
    let mut h1 = Some((heightmap1.ty, heightmap1.get_first_available(x, z)));
    let mut h2 = Some((heightmap2.ty, heightmap2.get_first_available(x, z)));
    let mut h3 = Some((heightmap3.ty, heightmap3.get_first_available(x, z)));
    let mut to_update = 4;

    for item in [&mut h0, &mut h1, &mut h2, &mut h3] {
        if let Some((_, height)) = item.as_ref() {
            if y + 2 <= *height {
                *item = None;
                to_update -= 1;
            }
        }
    }

    if to_update == 0 {
        return;
    }

    apply_direct(heightmap0, x, y, z, state, &mut h0, &mut to_update);
    apply_direct(heightmap1, x, y, z, state, &mut h1, &mut to_update);
    apply_direct(heightmap2, x, y, z, state, &mut h2, &mut to_update);
    apply_direct(heightmap3, x, y, z, state, &mut h3, &mut to_update);

    if to_update == 0 {
        return;
    }

    for search_y in (MIN_Y..y).rev() {
        let block_state = chunk.get_block_state(x, search_y, z);
        apply_search(heightmap0, x, z, search_y, block_state, &mut h0, &mut to_update);
        apply_search(heightmap1, x, z, search_y, block_state, &mut h1, &mut to_update);
        apply_search(heightmap2, x, z, search_y, block_state, &mut h2, &mut to_update);
        apply_search(heightmap3, x, z, search_y, block_state, &mut h3, &mut to_update);
        if to_update == 0 {
            break;
        }
    }

    if h0.is_some() {
        heightmap0.set_height(x, z, MIN_Y);
    }
    if h1.is_some() {
        heightmap1.set_height(x, z, MIN_Y);
    }
    if h2.is_some() {
        heightmap2.set_height(x, z, MIN_Y);
    }
    if h3.is_some() {
        heightmap3.set_height(x, z, MIN_Y);
    }
}

fn apply_direct(
    heightmap: &mut Heightmap,
    x: i32,
    y: i32,
    z: i32,
    state: State,
    slot: &mut Option<(Type, i32)>,
    to_update: &mut i32,
) {
    if let Some((_, height)) = *slot {
        if heightmap.is_opaque(state) {
            if y >= height {
                heightmap.set_height(x, z, y + 1);
            }
            *slot = None;
            *to_update -= 1;
        } else if height != y + 1 {
            *slot = None;
            *to_update -= 1;
        }
    }
}

fn apply_search(
    heightmap: &mut Heightmap,
    x: i32,
    z: i32,
    search_y: i32,
    state: State,
    slot: &mut Option<(Type, i32)>,
    to_update: &mut i32,
) {
    if slot.is_some() && heightmap.is_opaque(state) {
        heightmap.set_height(x, z, search_y + 1);
        *slot = None;
        *to_update -= 1;
    }
}

#[derive(Clone, Copy)]
enum State {
    Air,
    Stone,
    Water,
    Leaves,
}

impl State {
    fn not_air(self) -> bool {
        !matches!(self, Self::Air)
    }

    fn blocks_motion(self) -> bool {
        matches!(self, Self::Stone)
    }

    fn fluid(self) -> bool {
        matches!(self, Self::Water)
    }

    fn leaves(self) -> bool {
        matches!(self, Self::Leaves)
    }
}

#[derive(Clone, Copy)]
enum Type {
    MotionBlocking,
    MotionBlockingNoLeaves,
    OceanFloor,
    WorldSurface,
}

struct Chunk {
    columns: Vec<[State; HEIGHT]>,
}

impl Chunk {
    fn new() -> Self {
        let mut columns = Vec::with_capacity(COLUMN_COUNT);
        for column in 0..COLUMN_COUNT {
            let mut blocks = [State::Air; HEIGHT];
            let column_shift = (column & 3) as i32;
            let stone_top = 58 + column_shift;
            let water_y = 66 + (column_shift & 1);
            let leaves_y = 70 + column_shift;
            for y in MIN_Y..(MIN_Y + HEIGHT as i32) {
                let index = (y - MIN_Y) as usize;
                blocks[index] = if y <= stone_top {
                    State::Stone
                } else if y == water_y {
                    State::Water
                } else if y == leaves_y {
                    State::Leaves
                } else {
                    State::Air
                };
            }
            columns.push(blocks);
        }
        Self { columns }
    }

    fn get_block_state(&self, x: i32, y: i32, z: i32) -> State {
        self.columns[(x + (z << 4)) as usize][(y - MIN_Y) as usize]
    }
}

struct Heightmap {
    ty: Type,
    first_available: [i32; COLUMN_COUNT],
}

impl Heightmap {
    fn new(ty: Type) -> Self {
        Self {
            ty,
            first_available: [0; COLUMN_COUNT],
        }
    }

    fn update(&mut self, chunk: &Chunk, x: i32, y: i32, z: i32, state: State) -> bool {
        let first_available = self.get_first_available(x, z);
        if y <= first_available - 2 {
            return false;
        }

        if self.is_opaque(state) {
            if y >= first_available {
                self.set_height(x, z, y + 1);
                return true;
            }
        } else if first_available - 1 == y {
            for search_y in (MIN_Y..y).rev() {
                if self.is_opaque(chunk.get_block_state(x, search_y, z)) {
                    self.set_height(x, z, search_y + 1);
                    return true;
                }
            }
            self.set_height(x, z, MIN_Y);
            return true;
        }

        false
    }

    fn get_first_available(&self, x: i32, z: i32) -> i32 {
        self.first_available[(x + z * 16) as usize]
    }

    fn set_height(&mut self, x: i32, z: i32, value: i32) {
        self.first_available[(x + z * 16) as usize] = value;
    }

    fn is_opaque(&self, state: State) -> bool {
        match self.ty {
            Type::MotionBlocking => state.blocks_motion() || state.fluid(),
            Type::MotionBlockingNoLeaves => (state.blocks_motion() || state.fluid()) && !state.leaves(),
            Type::OceanFloor => state.blocks_motion(),
            Type::WorldSurface => state.not_air(),
        }
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
    fn old_and_new_match() {
        let old = old_four_update_summary(8);
        let new = new_combined_update_summary(8);
        assert_eq!(old.total, new.total);
    }
}
