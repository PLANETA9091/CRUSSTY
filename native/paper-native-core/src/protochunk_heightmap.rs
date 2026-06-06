#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtoChunkHeightmapError {
    InvalidInputLength,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

const TYPES: [Type; 6] = [
    Type::WorldSurfaceWg,
    Type::WorldSurface,
    Type::OceanFloorWg,
    Type::OceanFloor,
    Type::MotionBlocking,
    Type::MotionBlockingNoLeaves,
];

const SETS: [&[Type]; 4] = [
    &[Type::WorldSurfaceWg, Type::OceanFloorWg],
    &[Type::WorldSurfaceWg, Type::OceanFloorWg, Type::MotionBlocking],
    &[Type::OceanFloor, Type::MotionBlocking, Type::MotionBlockingNoLeaves],
    &TYPES,
];

#[derive(Clone, Copy)]
struct Heightmap {
    value: i64,
}

impl Heightmap {
    #[inline]
    fn update(&self, seed: usize) -> i64 {
        self.value + ((seed as i64) & 7)
    }
}

#[inline]
pub fn old_enumset_foreach_summary(iterations: usize) -> Result<i64, ProtoChunkHeightmapError> {
    run_summary(iterations, Mode::Old)
}

#[inline]
pub fn new_cached_contains_summary(iterations: usize) -> Result<i64, ProtoChunkHeightmapError> {
    run_summary(iterations, Mode::New)
}

#[derive(Clone, Copy)]
enum Mode {
    Old,
    New,
}

fn run_summary(iterations: usize, mode: Mode) -> Result<i64, ProtoChunkHeightmapError> {
    if iterations == 0 {
        return Ok(0);
    }

    let heightmaps = [
        Heightmap { value: 1 },
        Heightmap { value: 2 },
        Heightmap { value: 3 },
        Heightmap { value: 4 },
        Heightmap { value: 5 },
        Heightmap { value: 6 },
    ];

    let mut result = 0i64;
    for i in 0..iterations {
        let set = SETS[i & 3];
        let mut missing = 0usize;

        match mode {
            Mode::Old => {
                for &type_ in set {
                    if lookup(&heightmaps, type_).is_none() {
                        missing += 1;
                    }
                }

                result += missing as i64;

                for &type_ in set {
                    result += lookup(&heightmaps, type_).unwrap().update(i);
                }
            }
            Mode::New => {
                for &type_ in &TYPES {
                    if contains(set, type_) && lookup(&heightmaps, type_).is_none() {
                        missing += 1;
                    }
                }

                result += missing as i64;

                for &type_ in &TYPES {
                    if contains(set, type_) {
                        result += lookup(&heightmaps, type_).unwrap().update(i);
                    }
                }
            }
        }
    }

    Ok(result)
}

#[inline]
fn contains(set: &[Type], needle: Type) -> bool {
    set.iter().copied().any(|type_| type_ == needle)
}

#[inline]
fn lookup(heightmaps: &[Heightmap; 6], type_: Type) -> Option<Heightmap> {
    let index = match type_ {
        Type::WorldSurfaceWg => 0,
        Type::WorldSurface => 1,
        Type::OceanFloorWg => 2,
        Type::OceanFloor => 3,
        Type::MotionBlocking => 4,
        Type::MotionBlockingNoLeaves => 5,
    };
    Some(heightmaps[index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_and_new_match_on_iterations() {
        assert_eq!(old_enumset_foreach_summary(1).unwrap(), new_cached_contains_summary(1).unwrap());
        assert_eq!(old_enumset_foreach_summary(32).unwrap(), new_cached_contains_summary(32).unwrap());
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(old_enumset_foreach_summary(0).unwrap(), 0);
        assert_eq!(new_cached_contains_summary(0).unwrap(), 0);
    }
}
