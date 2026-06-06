use crate::position;

pub const NOT_SET: i32 = i32::MIN;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AreaOp {
    Add,
    Remove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AreaUpdateOp {
    pub op: AreaOp,
    pub chunk_x: i32,
    pub chunk_z: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AreaMapError {
    NegativeDistance,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AreaUpdateSummary {
    pub add_count: u64,
    pub remove_count: u64,
    pub add_checksum: u64,
    pub remove_checksum: u64,
    pub order_checksum: u64,
}

impl AreaUpdateSummary {
    #[inline]
    pub fn record(&mut self, op: AreaOp, chunk_x: i32, chunk_z: i32) {
        let packed = position::chunk_as_long(chunk_x, chunk_z) as u64;
        let op_tag = match op {
            AreaOp::Add => 0x9E37_79B9_7F4A_7C15u64,
            AreaOp::Remove => 0xC2B2_AE3D_27D4_EB4Fu64,
        };
        let mixed = mix64(
            packed
                ^ op_tag
                ^ ((chunk_x as u32 as u64) << 17)
                ^ ((chunk_z as u32 as u64).rotate_left(31)),
        );

        match op {
            AreaOp::Add => {
                self.add_count += 1;
                self.add_checksum = mix64(self.add_checksum ^ mixed);
            }
            AreaOp::Remove => {
                self.remove_count += 1;
                self.remove_checksum = mix64(self.remove_checksum ^ mixed);
            }
        }

        self.order_checksum = mix64(
            self.order_checksum
                ^ mixed
                ^ (self.add_count.wrapping_mul(0x1000_0000_01B3))
                ^ (self.remove_count.wrapping_mul(0xC6A4_A793_5BD1_E995)),
        );
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

#[inline]
fn sign(value: i32) -> i32 {
    1 | (value >> (i32::BITS - 1))
}

#[inline]
fn emit_square<F>(
    op: AreaOp,
    chunk_x: i32,
    chunk_z: i32,
    distance: i32,
    callback: &mut F,
) where
    F: FnMut(AreaOp, i32, i32),
{
    let max_x = chunk_x + distance;
    let max_z = chunk_z + distance;
    let mut cx = chunk_x - distance;
    while cx <= max_x {
        let mut cz = chunk_z - distance;
        while cz <= max_z {
            callback(op, cx, cz);
            cz += 1;
        }
        cx += 1;
    }
}

pub fn for_each_square<F>(
    op: AreaOp,
    chunk_x: i32,
    chunk_z: i32,
    distance: i32,
    mut callback: F,
) -> Result<(), AreaMapError>
where
    F: FnMut(AreaOp, i32, i32),
{
    if distance < 0 {
        return Err(AreaMapError::NegativeDistance);
    }

    emit_square(op, chunk_x, chunk_z, distance, &mut callback);
    Ok(())
}

pub fn for_each_update<F>(
    from_x: i32,
    from_z: i32,
    old_distance: i32,
    to_x: i32,
    to_z: i32,
    new_distance: i32,
    mut callback: F,
) -> Result<bool, AreaMapError>
where
    F: FnMut(AreaOp, i32, i32),
{
    if new_distance < 0 || old_distance < 0 {
        return Err(AreaMapError::NegativeDistance);
    }
    if from_x == NOT_SET {
        return Ok(false);
    }
    if from_x == to_x && from_z == to_z && old_distance == new_distance {
        return Ok(true);
    }

    let dx = to_x - from_x;
    let dz = to_z - from_z;
    let total_x = (from_x - to_x).abs();
    let total_z = (from_z - to_z).abs();

    if total_x.max(total_z) > (2 * new_distance.max(old_distance)) {
        emit_square(AreaOp::Remove, from_x, from_z, old_distance, &mut callback);
        emit_square(AreaOp::Add, to_x, to_z, new_distance, &mut callback);
        return Ok(true);
    }

    if old_distance != new_distance {
        let old_min_x = from_x - old_distance;
        let old_min_z = from_z - old_distance;
        let old_max_x = from_x + old_distance;
        let old_max_z = from_z + old_distance;
        let mut curr_x = old_min_x;
        while curr_x <= old_max_x {
            let x_distance = (curr_x - to_x).abs();
            let mut curr_z = old_min_z;
            while curr_z <= old_max_z {
                if x_distance.max((curr_z - to_z).abs()) > new_distance {
                    callback(AreaOp::Remove, curr_x, curr_z);
                }
                curr_z += 1;
            }
            curr_x += 1;
        }

        let new_min_x = to_x - new_distance;
        let new_min_z = to_z - new_distance;
        let new_max_x = to_x + new_distance;
        let new_max_z = to_z + new_distance;
        let mut curr_x = new_min_x;
        while curr_x <= new_max_x {
            let x_distance = (curr_x - from_x).abs();
            let mut curr_z = new_min_z;
            while curr_z <= new_max_z {
                if x_distance.max((curr_z - from_z).abs()) > old_distance {
                    callback(AreaOp::Add, curr_x, curr_z);
                }
                curr_z += 1;
            }
            curr_x += 1;
        }

        return Ok(true);
    }

    let up = sign(dz);
    let right = sign(dx);
    let distance = old_distance;

    if dx != 0 {
        let max_x = to_x + (distance * right) + right;
        let min_x = from_x + (distance * right) + right;
        let max_z = from_z + (distance * up) + up;
        let min_z = to_z - (distance * up);

        let mut curr_x = min_x;
        while curr_x != max_x {
            let mut curr_z = min_z;
            while curr_z != max_z {
                callback(AreaOp::Add, curr_x, curr_z);
                curr_z += up;
            }
            curr_x += right;
        }
    }

    if dz != 0 {
        let max_x = to_x + (distance * right) + right;
        let min_x = to_x - (distance * right);
        let max_z = to_z + (distance * up) + up;
        let min_z = from_z + (distance * up) + up;

        let mut curr_x = min_x;
        while curr_x != max_x {
            let mut curr_z = min_z;
            while curr_z != max_z {
                callback(AreaOp::Add, curr_x, curr_z);
                curr_z += up;
            }
            curr_x += right;
        }
    }

    if dx != 0 {
        let max_x = to_x - (distance * right);
        let min_x = from_x - (distance * right);
        let max_z = from_z + (distance * up) + up;
        let min_z = to_z - (distance * up);

        let mut curr_x = min_x;
        while curr_x != max_x {
            let mut curr_z = min_z;
            while curr_z != max_z {
                callback(AreaOp::Remove, curr_x, curr_z);
                curr_z += up;
            }
            curr_x += right;
        }
    }

    if dz != 0 {
        let max_x = from_x + (distance * right) + right;
        let min_x = from_x - (distance * right);
        let max_z = to_z - (distance * up);
        let min_z = from_z - (distance * up);

        let mut curr_x = min_x;
        while curr_x != max_x {
            let mut curr_z = min_z;
            while curr_z != max_z {
                callback(AreaOp::Remove, curr_x, curr_z);
                curr_z += up;
            }
            curr_x += right;
        }
    }

    Ok(true)
}

pub fn summarize_update(
    from_x: i32,
    from_z: i32,
    old_distance: i32,
    to_x: i32,
    to_z: i32,
    new_distance: i32,
) -> Result<AreaUpdateSummary, AreaMapError> {
    let mut summary = AreaUpdateSummary::default();
    for_each_update(
        from_x,
        from_z,
        old_distance,
        to_x,
        to_z,
        new_distance,
        |op, chunk_x, chunk_z| summary.record(op, chunk_x, chunk_z),
    )?;
    Ok(summary)
}

pub fn summarize_square(
    op: AreaOp,
    chunk_x: i32,
    chunk_z: i32,
    distance: i32,
) -> Result<AreaUpdateSummary, AreaMapError> {
    let mut summary = AreaUpdateSummary::default();
    for_each_square(op, chunk_x, chunk_z, distance, |op, chunk_x, chunk_z| {
        summary.record(op, chunk_x, chunk_z)
    })?;
    Ok(summary)
}

pub fn collect_update_ops(
    from_x: i32,
    from_z: i32,
    old_distance: i32,
    to_x: i32,
    to_z: i32,
    new_distance: i32,
) -> Result<Vec<AreaUpdateOp>, AreaMapError> {
    let mut ret = Vec::new();
    for_each_update(
        from_x,
        from_z,
        old_distance,
        to_x,
        to_z,
        new_distance,
        |op, chunk_x, chunk_z| ret.push(AreaUpdateOp { op, chunk_x, chunk_z }),
    )?;
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn collect_update(
        from_x: i32,
        from_z: i32,
        old_distance: i32,
        to_x: i32,
        to_z: i32,
        new_distance: i32,
    ) -> Vec<(AreaOp, i32, i32)> {
        let mut ret = Vec::new();
        for_each_update(
            from_x,
            from_z,
            old_distance,
            to_x,
            to_z,
            new_distance,
            |op, chunk_x, chunk_z| ret.push((op, chunk_x, chunk_z)),
        )
        .unwrap();
        ret
    }

    fn collect_square(
        op: AreaOp,
        chunk_x: i32,
        chunk_z: i32,
        distance: i32,
    ) -> Vec<(AreaOp, i32, i32)> {
        let mut ret = Vec::new();
        for_each_square(op, chunk_x, chunk_z, distance, |op, chunk_x, chunk_z| {
            ret.push((op, chunk_x, chunk_z))
        })
        .unwrap();
        ret
    }

    fn square(center_x: i32, center_z: i32, distance: i32) -> HashSet<(i32, i32)> {
        let mut ret = HashSet::new();
        for x in (center_x - distance)..=(center_x + distance) {
            for z in (center_z - distance)..=(center_z + distance) {
                ret.insert((x, z));
            }
        }
        ret
    }

    #[test]
    fn unchanged_position_emits_no_callbacks() {
        let events = collect_update(4, -3, 2, 4, -3, 2);
        assert!(events.is_empty());
        assert_eq!(summarize_update(4, -3, 2, 4, -3, 2).unwrap(), AreaUpdateSummary::default());
    }

    #[test]
    fn unset_source_reports_not_updated() {
        let mut called = false;
        assert_eq!(
            for_each_update(NOT_SET, 0, 1, 0, 0, 1, |_, _, _| called = true),
            Ok(false)
        );
        assert!(!called);
    }

    #[test]
    fn single_step_right_matches_java_order() {
        let events = collect_update(0, 0, 1, 1, 0, 1);
        assert_eq!(
            events,
            vec![
                (AreaOp::Add, 2, -1),
                (AreaOp::Add, 2, 0),
                (AreaOp::Add, 2, 1),
                (AreaOp::Remove, -1, -1),
                (AreaOp::Remove, -1, 0),
                (AreaOp::Remove, -1, 1),
            ]
        );
    }

    #[test]
    fn square_add_matches_java_order() {
        let events = collect_square(AreaOp::Add, 4, -2, 1);
        assert_eq!(
            events,
            vec![
                (AreaOp::Add, 3, -3),
                (AreaOp::Add, 3, -2),
                (AreaOp::Add, 3, -1),
                (AreaOp::Add, 4, -3),
                (AreaOp::Add, 4, -2),
                (AreaOp::Add, 4, -1),
                (AreaOp::Add, 5, -3),
                (AreaOp::Add, 5, -2),
                (AreaOp::Add, 5, -1),
            ]
        );
    }

    #[test]
    fn square_remove_summary_matches_manual_recording() {
        let events = collect_square(AreaOp::Remove, -1, 2, 2);
        let mut expected = AreaUpdateSummary::default();
        for (_, chunk_x, chunk_z) in &events {
            expected.record(AreaOp::Remove, *chunk_x, *chunk_z);
        }

        assert_eq!(events.len(), 25);
        assert_eq!(summarize_square(AreaOp::Remove, -1, 2, 2), Ok(expected));
    }

    #[test]
    fn square_rejects_negative_distance() {
        let mut called = false;
        assert_eq!(
            for_each_square(AreaOp::Add, 0, 0, -1, |_, _, _| called = true),
            Err(AreaMapError::NegativeDistance)
        );
        assert!(!called);
    }

    #[test]
    fn update_rejects_negative_distance() {
        let mut called = false;
        assert_eq!(
            for_each_update(0, 0, -1, 1, 1, 2, |_, _, _| called = true),
            Err(AreaMapError::NegativeDistance)
        );
        assert_eq!(
            for_each_update(0, 0, 1, 1, 1, -2, |_, _, _| called = true),
            Err(AreaMapError::NegativeDistance)
        );
        assert!(!called);
    }

    #[test]
    fn diagonal_step_matches_naive_set_difference() {
        let events = collect_update(0, 0, 2, 1, 1, 2);
        assert_set_difference(0, 0, 2, 1, 1, 2, &events);
        assert_eq!(events.first(), Some(&(AreaOp::Add, 3, -1)));
        assert_eq!(events.last(), Some(&(AreaOp::Remove, 2, -2)));
    }

    #[test]
    fn teleport_removes_old_before_adding_new() {
        let events = collect_update(0, 0, 1, 9, 0, 1);
        assert_eq!(events.len(), 18);
        assert!(events[..9].iter().all(|event| event.0 == AreaOp::Remove));
        assert!(events[9..].iter().all(|event| event.0 == AreaOp::Add));
        assert_set_difference(0, 0, 1, 9, 0, 1, &events);
    }

    #[test]
    fn distance_change_matches_naive_set_difference() {
        let events = collect_update(-2, 3, 1, -1, 2, 3);
        assert_set_difference(-2, 3, 1, -1, 2, 3, &events);
    }

    #[test]
    fn small_grid_matches_naive_set_difference() {
        for from_x in -2..=2 {
            for from_z in -2..=2 {
                for to_x in -2..=2 {
                    for to_z in -2..=2 {
                        for old_distance in 0..=4 {
                            for new_distance in 0..=4 {
                                let events = collect_update(
                                    from_x,
                                    from_z,
                                    old_distance,
                                    to_x,
                                    to_z,
                                    new_distance,
                                );
                                assert_set_difference(
                                    from_x,
                                    from_z,
                                    old_distance,
                                    to_x,
                                    to_z,
                                    new_distance,
                                    &events,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn assert_set_difference(
        from_x: i32,
        from_z: i32,
        old_distance: i32,
        to_x: i32,
        to_z: i32,
        new_distance: i32,
        events: &[(AreaOp, i32, i32)],
    ) {
        let old_set = square(from_x, from_z, old_distance);
        let new_set = square(to_x, to_z, new_distance);
        let expected_adds: HashSet<(i32, i32)> = new_set.difference(&old_set).copied().collect();
        let expected_removes: HashSet<(i32, i32)> = old_set.difference(&new_set).copied().collect();
        let mut actual_adds = HashSet::new();
        let mut actual_removes = HashSet::new();

        for &(op, x, z) in events {
            match op {
                AreaOp::Add => assert!(actual_adds.insert((x, z))),
                AreaOp::Remove => assert!(actual_removes.insert((x, z))),
            }
        }

        assert_eq!(actual_adds, expected_adds);
        assert_eq!(actual_removes, expected_removes);
    }
}
