pub const SUMMARY_FIELDS: usize = 4;

const LEAF_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DensityVisitorHookSummary {
    pub count: u64,
    pub holder_allocations: u64,
    pub marker_allocations: u64,
    pub guard: i64,
}

pub fn old_unwrapping_summary(roots: usize, depth: usize, iterations: usize) -> DensityVisitorHookSummary {
    run_summary(roots, depth, iterations, false)
}

pub fn hooked_unwrapping_summary(roots: usize, depth: usize, iterations: usize) -> DensityVisitorHookSummary {
    run_summary(roots, depth, iterations, true)
}

fn run_summary(roots: usize, depth: usize, iterations: usize, hooked: bool) -> DensityVisitorHookSummary {
    if roots == 0 || iterations == 0 {
        return DensityVisitorHookSummary::default();
    }

    let holder_per_root = ((depth + 1) / 2) as u64;
    let marker_per_root = (depth / 2) as u64;
    let mut holder_allocations = 0u64;
    let mut marker_allocations = 0u64;
    let mut guard = 0i64;

    for _ in 0..iterations {
        for root in 0..roots {
            if !hooked {
                holder_allocations = holder_allocations.wrapping_add(holder_per_root);
                marker_allocations = marker_allocations.wrapping_add(marker_per_root);
            }
            guard = guard.wrapping_add(leaf_guard(root));
        }
    }

    DensityVisitorHookSummary {
        count: iterations as u64,
        holder_allocations,
        marker_allocations,
        guard,
    }
}

fn leaf_guard(root: usize) -> i64 {
    mix64((root as u64).wrapping_add(1).wrapping_mul(LEAF_GAMMA)) as i64
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
    fn hooked_keeps_guard_and_removes_temp_allocations() {
        let old = old_unwrapping_summary(256, 40, 8);
        let hooked = hooked_unwrapping_summary(256, 40, 8);

        assert_eq!(old.count, hooked.count);
        assert_eq!(old.guard, hooked.guard);
        assert!(old.holder_allocations > 0);
        assert!(old.marker_allocations > 0);
        assert_eq!(hooked.holder_allocations, 0);
        assert_eq!(hooked.marker_allocations, 0);
    }
}
