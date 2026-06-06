pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const DEPENDENCIES: [Status; 9] = [
    Status::Features,
    Status::Carvers,
    Status::Noise,
    Status::Surface,
    Status::Surface,
    Status::Biomes,
    Status::Biomes,
    Status::StructureStarts,
    Status::Empty,
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChunkDependenciesSummary {
    pub count: u64,
    pub value: i64,
    pub checksum: u64,
    pub last_value: i64,
}

pub fn old_immutable_list_summary(iterations: usize) -> ChunkDependenciesSummary {
    let dependencies = OldDependencies::new();
    run_summary(iterations, |radius| dependencies.get(radius))
}

pub fn array_summary(iterations: usize) -> ChunkDependenciesSummary {
    let dependencies = ArrayDependencies::new();
    run_summary(iterations, |radius| dependencies.get(radius))
}

fn run_summary<F>(iterations: usize, mut get: F) -> ChunkDependenciesSummary
where
    F: FnMut(usize) -> Status,
{
    if iterations == 0 {
        return ChunkDependenciesSummary::default();
    }

    let size = DEPENDENCIES.len();
    let mut value = 0i32;
    let mut checksum = 0u64;
    let mut last_value = 0i32;

    for iteration in 0..iterations {
        let distance = dependency_distance(iteration, size);
        if distance < size {
            let index = get(distance).index();
            value = value.wrapping_add(index);
            last_value = index;
            checksum = mix64(
                checksum
                    ^ (index as u32 as u64)
                    ^ ((distance as u64) << 8)
                    ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)),
            );
        } else {
            last_value = -1;
            checksum = mix64(
                checksum
                    ^ 0xFFFF_FFFFu64
                    ^ ((distance as u64) << 8)
                    ^ ((iteration as u64).wrapping_mul(MIX_GAMMA)),
            );
        }
    }

    ChunkDependenciesSummary {
        count: iterations as u64,
        value: i64::from(value),
        checksum,
        last_value: i64::from(last_value),
    }
}

#[inline]
fn dependency_distance(iteration: usize, size: usize) -> usize {
    let mixed = (iteration as i32).wrapping_mul(0x9E37_79B9u32 as i32) as u32;
    ((mixed >> 1) as usize) % (size + 2)
}

#[derive(Clone)]
#[allow(dead_code)]
struct OldDependencies {
    dependency_by_radius: Vec<Status>,
    radius_by_dependency: Vec<usize>,
}

#[allow(dead_code)]
impl OldDependencies {
    fn new() -> Self {
        let dependency_by_radius = DEPENDENCIES.to_vec();
        let radius_length = dependency_by_radius[0].index() as usize + 1;
        let mut radius_by_dependency = vec![0usize; radius_length];

        for (radius, status) in dependency_by_radius.iter().enumerate() {
            for index in 0..=status.index() as usize {
                radius_by_dependency[index] = radius;
            }
        }

        Self {
            dependency_by_radius,
            radius_by_dependency,
        }
    }

    #[inline]
    fn size(&self) -> usize {
        self.dependency_by_radius.len()
    }

    #[inline]
    fn radius(&self) -> usize {
        self.size().saturating_sub(1)
    }

    #[inline]
    fn radius_of(&self, status: Status) -> usize {
        self.radius_by_dependency[status.index() as usize]
    }

    #[inline]
    fn get(&self, radius: usize) -> Status {
        self.dependency_by_radius[radius]
    }
}

#[allow(dead_code)]
struct ArrayDependencies {
    dependency_by_radius: [Status; 9],
    radius_by_dependency: [usize; 7],
    size: usize,
    radius: usize,
}

#[allow(dead_code)]
impl ArrayDependencies {
    fn new() -> Self {
        let dependency_by_radius = DEPENDENCIES;
        let size = dependency_by_radius.len();
        let radius = size.saturating_sub(1);
        let mut radius_by_dependency = [0usize; 7];

        for (radius, status) in dependency_by_radius.iter().enumerate() {
            for index in 0..=status.index() as usize {
                radius_by_dependency[index] = radius;
            }
        }

        Self {
            dependency_by_radius,
            radius_by_dependency,
            size,
            radius,
        }
    }

    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    #[inline]
    fn radius(&self) -> usize {
        self.radius
    }

    #[inline]
    fn radius_of(&self, status: Status) -> usize {
        self.radius_by_dependency[status.index() as usize]
    }

    #[inline]
    fn get(&self, radius: usize) -> Status {
        self.dependency_by_radius[radius]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    Empty,
    StructureStarts,
    Biomes,
    Surface,
    Noise,
    Carvers,
    Features,
}

impl Status {
    #[allow(dead_code)]
    const ALL: [Self; 7] = [
        Self::Empty,
        Self::StructureStarts,
        Self::Biomes,
        Self::Surface,
        Self::Noise,
        Self::Carvers,
        Self::Features,
    ];

    #[inline]
    fn index(self) -> i32 {
        match self {
            Self::Empty => 0,
            Self::StructureStarts => 1,
            Self::Biomes => 2,
            Self::Surface => 3,
            Self::Noise => 4,
            Self::Carvers => 5,
            Self::Features => 6,
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
    fn old_and_array_summaries_match() {
        let old = old_immutable_list_summary(128_000);
        let array = array_summary(128_000);

        assert_eq!(old, array);
        assert_eq!(old.count, 128_000);
    }

    #[test]
    fn dependency_metadata_matches() {
        let old = OldDependencies::new();
        let array = ArrayDependencies::new();

        assert_eq!(old.size(), array.size());
        assert_eq!(old.radius(), array.radius());
        for radius in 0..old.size() {
            assert_eq!(old.get(radius), array.get(radius));
        }
        for status in Status::ALL {
            assert_eq!(old.radius_of(status), array.radius_of(status));
        }
    }

    #[test]
    fn java_distance_sequence_is_stable() {
        let expected = [0usize, 3, 5, 8, 10, 0, 4, 5, 9, 10, 1, 4];
        for (iteration, distance) in expected.iter().copied().enumerate() {
            assert_eq!(dependency_distance(iteration, DEPENDENCIES.len()), distance);
        }
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(
            old_immutable_list_summary(0),
            ChunkDependenciesSummary::default()
        );
    }
}
