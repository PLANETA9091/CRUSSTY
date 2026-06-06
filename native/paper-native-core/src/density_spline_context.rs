pub const SUMMARY_FIELDS: usize = 4;

const MIX_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const LOCATIONS: [f32; 4] = [0.0, 0.35, 0.7, 1.0];
const DERIVATIVES: [f32; 4] = [0.05, 0.0, -0.03, 0.01];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DensitySplineContextSummary {
    pub count: u64,
    pub sum_bits: u64,
    pub checksum: u64,
    pub last_bits: u64,
}

#[derive(Clone, Copy)]
struct Context {
    block_x: i32,
    block_y: i32,
    block_z: i32,
}

#[derive(Clone, Copy)]
struct Point {
    context: Context,
}

#[derive(Clone, Copy)]
enum Function {
    A,
    B,
    C,
    D,
}

pub fn old_wrapper_summary(iterations: usize) -> DensitySplineContextSummary {
    run_summary(iterations, Mode::OldWrapper)
}

pub fn new_direct_summary(iterations: usize) -> DensitySplineContextSummary {
    run_summary(iterations, Mode::NewDirect)
}

#[derive(Clone, Copy)]
enum Mode {
    OldWrapper,
    NewDirect,
}

fn run_summary(iterations: usize, mode: Mode) -> DensitySplineContextSummary {
    if iterations == 0 {
        return DensitySplineContextSummary::default();
    }

    let mut sum = 0.0f64;
    let mut checksum = 0u64;
    let mut last_bits = 0u64;
    for i in 0..iterations {
        let context = Context {
            block_x: ((i * 37) & 1023) as i32,
            block_y: ((i * 17) & 255) as i32,
            block_z: ((i * 29) & 1023) as i32,
        };
        let value = match mode {
            Mode::OldWrapper => old_compute(context),
            Mode::NewDirect => new_compute(context),
        };
        sum += value;
        last_bits = value.to_bits();
        checksum = mix64(checksum ^ last_bits ^ ((i as u64).wrapping_mul(MIX_GAMMA)));
    }

    DensitySplineContextSummary {
        count: iterations as u64,
        sum_bits: sum.to_bits(),
        checksum,
        last_bits,
    }
}

fn old_compute(context: Context) -> f64 {
    old_apply(Point { context })
}

fn old_apply(point: Point) -> f64 {
    let f = coordinate(point.context);
    let index = find_interval_start(&LOCATIONS, f);
    if index < 0 {
        return linear_extend(f, &LOCATIONS, compute(Function::A, point.context) as f32, &DERIVATIVES, 0) as f64;
    }
    if index == LOCATIONS.len() as i32 - 1 {
        return linear_extend(
            f,
            &LOCATIONS,
            compute(Function::D, point.context) as f32,
            &DERIVATIVES,
            LOCATIONS.len() - 1,
        ) as f64;
    }
    spline_middle(f, index as usize, point.context) as f64
}

fn new_compute(context: Context) -> f64 {
    let f = coordinate(context);
    let index = find_interval_start(&LOCATIONS, f);
    if index < 0 {
        return linear_extend(f, &LOCATIONS, compute(Function::A, context) as f32, &DERIVATIVES, 0) as f64;
    }
    if index == LOCATIONS.len() as i32 - 1 {
        return linear_extend(
            f,
            &LOCATIONS,
            compute(Function::D, context) as f32,
            &DERIVATIVES,
            LOCATIONS.len() - 1,
        ) as f64;
    }
    spline_middle(f, index as usize, context) as f64
}

fn spline_middle(f: f32, index: usize, context: Context) -> f32 {
    let f1 = LOCATIONS[index];
    let f2 = LOCATIONS[index + 1];
    let delta = (f - f1) / (f2 - f1);
    let v0 = compute(Function::A, context) as f32;
    let v1 = compute(Function::B, context) as f32;
    let v2 = compute(Function::C, context) as f32;
    let v3 = compute(Function::D, context) as f32;
    let a = DERIVATIVES[index];
    let b = DERIVATIVES[index + 1];
    let d0 = v1 - v0;
    let d1 = v2 - v1;
    let d2 = v3 - v2;
    let span = f2 - f1;
    let left = lerp(delta, v0, v1)
        + delta * (1.0 - delta) * lerp(delta, a * span - d0, -b * span + d0);
    let right = lerp(delta, v1, v2)
        + delta * (1.0 - delta) * lerp(delta, a * span - d1, -b * span + d1);
    let extra = lerp(delta, v2, v3)
        + delta * (1.0 - delta) * lerp(delta, a * span - d2, -b * span + d2);
    lerp3(
        delta, delta, delta, left, right, extra, extra, left, right, extra, extra,
    )
}

fn coordinate(context: Context) -> f32 {
    (context.block_x as f64 * 0.0015
        + context.block_y as f64 * 0.003
        + context.block_z as f64 * 0.002) as f32
}

fn compute(function: Function, context: Context) -> f64 {
    let (scale_x, scale_y, scale_z, offset) = match function {
        Function::A => (0.017, 0.021, 0.013, 0.1),
        Function::B => (-0.013, 0.029, 0.019, -0.2),
        Function::C => (0.023, -0.011, 0.031, 0.3),
        Function::D => (0.007, 0.013, -0.017, 0.4),
    };
    context.block_x as f64 * scale_x
        + context.block_y as f64 * scale_y
        + context.block_z as f64 * scale_z
        + offset
}

fn find_interval_start(locations: &[f32], start: f32) -> i32 {
    let mut low = 0usize;
    let mut high = locations.len();
    while low < high {
        let mid = (low + high) >> 1;
        if start < locations[mid] {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    low as i32 - 1
}

fn linear_extend(coordinate: f32, locations: &[f32], value: f32, derivatives: &[f32], index: usize) -> f32 {
    let derivative = derivatives[index];
    if derivative == 0.0 {
        value
    } else {
        value + derivative * (coordinate - locations[index])
    }
}

#[inline]
fn lerp(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

#[inline]
fn lerp2(delta_x: f32, delta_y: f32, value00: f32, value10: f32, value01: f32, value11: f32) -> f32 {
    lerp(delta_y, lerp(delta_x, value00, value10), lerp(delta_x, value01, value11))
}

#[allow(clippy::too_many_arguments)]
fn lerp3(
    delta_x: f32,
    delta_y: f32,
    delta_z: f32,
    value000: f32,
    value100: f32,
    value010: f32,
    value110: f32,
    value001: f32,
    value101: f32,
    value011: f32,
    value111: f32,
) -> f32 {
    lerp(
        delta_z,
        lerp2(delta_x, delta_y, value000, value100, value010, value110),
        lerp2(delta_x, delta_y, value001, value101, value011, value111),
    )
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
        assert_eq!(old_wrapper_summary(4096), new_direct_summary(4096));
    }

    #[test]
    fn zero_iterations_are_empty() {
        assert_eq!(old_wrapper_summary(0), DensitySplineContextSummary::default());
    }
}
