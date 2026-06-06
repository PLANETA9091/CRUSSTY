#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoiseGeneratorSettingsError {
    InvalidInputLength,
}

#[derive(Clone, Copy)]
struct GeneratorValues<'a> {
    sea_levels: &'a [i32],
    min_ys: &'a [i32],
    heights: &'a [i32],
    mask: usize,
}

#[derive(Clone, Copy)]
struct Settings {
    sea_level: i32,
    noise_settings: NoiseSettings,
}

#[derive(Clone, Copy)]
struct NoiseSettings {
    min_y: i32,
    height: i32,
}

#[derive(Clone, Copy)]
struct CachedValues {
    sea_level: i32,
    min_y: i32,
    height: i32,
}

#[inline]
pub fn holder_value_settings_sum(
    sea_levels: &[i32],
    min_ys: &[i32],
    heights: &[i32],
    iterations: usize,
) -> Result<i32, NoiseGeneratorSettingsError> {
    run_sum(sea_levels, min_ys, heights, iterations, Mode::HolderValue)
}

#[inline]
pub fn memoized_supplier_settings_sum(
    sea_levels: &[i32],
    min_ys: &[i32],
    heights: &[i32],
    iterations: usize,
) -> Result<i32, NoiseGeneratorSettingsError> {
    run_sum(sea_levels, min_ys, heights, iterations, Mode::MemoizedSupplier)
}

#[inline]
pub fn lazy_primitive_settings_sum(
    sea_levels: &[i32],
    min_ys: &[i32],
    heights: &[i32],
    iterations: usize,
) -> Result<i32, NoiseGeneratorSettingsError> {
    run_sum(sea_levels, min_ys, heights, iterations, Mode::LazyPrimitive)
}

#[inline]
pub fn manual_lazy_object_settings_sum(
    sea_levels: &[i32],
    min_ys: &[i32],
    heights: &[i32],
    iterations: usize,
) -> Result<i32, NoiseGeneratorSettingsError> {
    run_sum(sea_levels, min_ys, heights, iterations, Mode::ManualLazyObject)
}

#[inline]
pub fn cached_int_settings_sum(
    sea_levels: &[i32],
    min_ys: &[i32],
    heights: &[i32],
    iterations: usize,
) -> Result<i32, NoiseGeneratorSettingsError> {
    run_sum(sea_levels, min_ys, heights, iterations, Mode::CachedInt)
}

#[derive(Clone, Copy)]
enum Mode {
    HolderValue,
    MemoizedSupplier,
    LazyPrimitive,
    ManualLazyObject,
    CachedInt,
}

fn run_sum(
    sea_levels: &[i32],
    min_ys: &[i32],
    heights: &[i32],
    iterations: usize,
    mode: Mode,
) -> Result<i32, NoiseGeneratorSettingsError> {
    if sea_levels.is_empty()
        || sea_levels.len() != min_ys.len()
        || sea_levels.len() != heights.len()
    {
        return Err(NoiseGeneratorSettingsError::InvalidInputLength);
    }

    let data = GeneratorValues {
        sea_levels,
        min_ys,
        heights,
        mask: sea_levels.len() - 1,
    };

    let sum = match mode {
        Mode::HolderValue => data.holder_value_settings_sum(iterations),
        Mode::MemoizedSupplier => data.memoized_supplier_settings_sum(iterations),
        Mode::LazyPrimitive => data.lazy_primitive_settings_sum(iterations),
        Mode::ManualLazyObject => data.manual_lazy_object_settings_sum(iterations),
        Mode::CachedInt => data.cached_int_settings_sum(iterations),
    };

    Ok(sum)
}

impl<'a> GeneratorValues<'a> {
    #[inline]
    fn holder_value_settings_sum(&self, iterations: usize) -> i32 {
        let mut sum = 0i32;
        for iteration in 0..iterations {
            let index = iteration & self.mask;
            sum = sum.wrapping_add(self.holder_value_sea_level(index));
            sum = sum.wrapping_add(self.holder_value_min_y(index));
            sum = sum.wrapping_add(self.holder_value_height(index));
        }
        sum
    }

    #[inline]
    fn memoized_supplier_settings_sum(&self, iterations: usize) -> i32 {
        let mut sum = 0i32;
        for iteration in 0..iterations {
            let index = iteration & self.mask;
            let settings = self.settings(index);
            sum = sum.wrapping_add(settings.sea_level);
            let noise_settings = settings.noise_settings;
            sum = sum.wrapping_add(noise_settings.min_y);
            sum = sum.wrapping_add(noise_settings.height);
        }
        sum
    }

    #[inline]
    fn lazy_primitive_settings_sum(&self, iterations: usize) -> i32 {
        let mut sum = 0i32;
        for iteration in 0..iterations {
            let index = iteration & self.mask;
            let sea_level = self.sea_levels[index];
            let min_y = self.min_ys[index];
            let height = self.heights[index];
            sum = sum.wrapping_add(sea_level);
            sum = sum.wrapping_add(min_y);
            sum = sum.wrapping_add(height);
        }
        sum
    }

    #[inline]
    fn manual_lazy_object_settings_sum(&self, iterations: usize) -> i32 {
        let mut sum = 0i32;
        for iteration in 0..iterations {
            let index = iteration & self.mask;
            let settings = self.settings(index);
            let noise_settings = self.noise_settings(index);
            sum = sum.wrapping_add(settings.sea_level);
            sum = sum.wrapping_add(noise_settings.min_y);
            sum = sum.wrapping_add(noise_settings.height);
        }
        sum
    }

    #[inline]
    fn cached_int_settings_sum(&self, iterations: usize) -> i32 {
        let mut sum = 0i32;
        for iteration in 0..iterations {
            let index = iteration & self.mask;
            let cached = self.cached_values(index);
            sum = sum.wrapping_add(cached.sea_level);
            sum = sum.wrapping_add(cached.min_y);
            sum = sum.wrapping_add(cached.height);
        }
        sum
    }

    #[inline]
    fn holder_value_sea_level(&self, index: usize) -> i32 {
        self.sea_levels[index]
    }

    #[inline]
    fn holder_value_min_y(&self, index: usize) -> i32 {
        self.min_ys[index]
    }

    #[inline]
    fn holder_value_height(&self, index: usize) -> i32 {
        self.heights[index]
    }

    #[inline]
    fn settings(&self, index: usize) -> Settings {
        Settings {
            sea_level: self.sea_levels[index],
            noise_settings: self.noise_settings(index),
        }
    }

    #[inline]
    fn noise_settings(&self, index: usize) -> NoiseSettings {
        NoiseSettings {
            min_y: self.min_ys[index],
            height: self.heights[index],
        }
    }

    #[inline]
    fn cached_values(&self, index: usize) -> CachedValues {
        CachedValues {
            sea_level: self.sea_levels[index],
            min_y: self.min_ys[index],
            height: self.heights[index],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_modes_match() {
        let sea_levels = [48, 49, 50, 51];
        let min_ys = [-64, -63, -62, -61];
        let heights = [384, 383, 382, 381];
        let iterations = 32usize;

        let holder = holder_value_settings_sum(&sea_levels, &min_ys, &heights, iterations).unwrap();
        let memoized = memoized_supplier_settings_sum(&sea_levels, &min_ys, &heights, iterations).unwrap();
        let lazy = lazy_primitive_settings_sum(&sea_levels, &min_ys, &heights, iterations).unwrap();
        let manual = manual_lazy_object_settings_sum(&sea_levels, &min_ys, &heights, iterations).unwrap();
        let cached = cached_int_settings_sum(&sea_levels, &min_ys, &heights, iterations).unwrap();

        assert_eq!(holder, memoized);
        assert_eq!(holder, lazy);
        assert_eq!(holder, manual);
        assert_eq!(holder, cached);
        assert_eq!(holder, 11_824);
    }

    #[test]
    fn rejects_mismatched_lengths() {
        let sea_levels = [48, 49];
        let min_ys = [-64];
        let heights = [384, 383];
        assert_eq!(
            holder_value_settings_sum(&sea_levels, &min_ys, &heights, 1),
            Err(NoiseGeneratorSettingsError::InvalidInputLength)
        );
    }
}
