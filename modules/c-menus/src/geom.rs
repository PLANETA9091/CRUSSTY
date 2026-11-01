//! Geometry helpers: yaw math, menu-space <-> world-space mapping,
//! ray/AABB intersection. Everything f64; yaws are Bukkit degrees
//! (0 = +Z south, 90 = -X west).

/// Horizontal basis vectors for a Bukkit yaw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Basis {
    /// Forward (the way the player looks, horizontal).
    pub fx: f64,
    pub fz: f64,
    /// Right (the player's right hand side).
    pub rx: f64,
    pub rz: f64,
}

impl Basis {
    pub fn from_yaw_deg(yaw: f64) -> Self {
        let y = yaw.to_radians();
        Basis {
            fx: -y.sin(),
            fz: y.cos(),
            rx: -y.cos(),
            rz: -y.sin(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    pub fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }

    pub fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }

    pub fn scale(self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }

    pub fn dot(self, o: Vec3) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn len(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Vec3 {
        let l = self.len();
        if l < 1e-9 {
            Vec3::ZERO
        } else {
            self.scale(1.0 / l)
        }
    }
}

/// A ray (origin + direction) and the menu-space basis it was built in.
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    /// Build a ray from eye position + yaw/pitch (Bukkit degrees).
    pub fn from_look(ex: f64, ey: f64, ez: f64, yaw_deg: f64, pitch_deg: f64) -> Ray {
        let y = yaw_deg.to_radians();
        let p = pitch_deg.to_radians();
        // Minecraft look vector: x = -sin(yaw)cos(pitch), y = -sin(pitch), z = cos(yaw)cos(pitch)
        let dir = Vec3::new(
            -y.sin() * p.cos(),
            -p.sin(),
            y.cos() * p.cos(),
        );
        Ray {
            origin: Vec3::new(ex, ey, ez),
            dir: dir.normalized(),
        }
    }

    /// Transform the ray into menu space: origin relative to `anchor`,
    /// x along basis.right, y world-up, z along basis.forward.
    pub fn to_menu_space(self, anchor: Vec3, b: Basis) -> Ray {
        let rel = self.origin.sub(anchor);
        let origin = Vec3::new(
            rel.x * b.rx + rel.z * b.rz,
            rel.y,
            rel.x * b.fx + rel.z * b.fz,
        );
        let dx = self.dir.x * b.rx + self.dir.z * b.rz;
        let dz = self.dir.x * b.fx + self.dir.z * b.fz;
        Ray {
            origin,
            dir: Vec3::new(dx, self.dir.y, dz),
        }
    }
}

/// Axis-aligned box in menu space (meters).
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn centered(cx: f64, cy: f64, cz: f64, w: f64, h: f64, d: f64) -> Aabb {
        Aabb {
            min: Vec3::new(cx - w / 2.0, cy - h / 2.0, cz - d / 2.0),
            max: Vec3::new(cx + w / 2.0, cy + h / 2.0, cz + d / 2.0),
        }
    }

    /// Slab test; returns distance along the ray or None.
    pub fn ray_hit(&self, r: Ray) -> Option<f64> {
        let mut tmin = -f64::INFINITY;
        let mut tmax = f64::INFINITY;
        for axis in 0..3 {
            let (o, d, mn, mx) = match axis {
                0 => (r.origin.x, r.dir.x, self.min.x, self.max.x),
                1 => (r.origin.y, r.dir.y, self.min.y, self.max.y),
                _ => (r.origin.z, r.dir.z, self.min.z, self.max.z),
            };
            if d.abs() < 1e-9 {
                if o < mn || o > mx {
                    return None;
                }
            } else {
                let mut t1 = (mn - o) / d;
                let mut t2 = (mx - o) / d;
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }
                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
                if tmin > tmax {
                    return None;
                }
            }
        }
        if tmax < 0.0 {
            return None;
        }
        Some(if tmin < 0.0 { 0.0 } else { tmin })
    }
}

/// World position of a menu-space point: anchor + right*x + up*y - forward*z
/// (positive z brings the element closer to the viewer).
pub fn menu_to_world(anchor: Vec3, b: Basis, mx: f64, my: f64, mz: f64) -> Vec3 {
    Vec3::new(
        anchor.x + b.rx * mx - b.fx * mz,
        anchor.y + my,
        anchor.z + b.rz * mx - b.fz * mz,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn basis_yaws() {
        // yaw 0: facing +Z, right = -X
        let b = Basis::from_yaw_deg(0.0);
        assert!(approx(b.fx, 0.0) && approx(b.fz, 1.0));
        assert!(approx(b.rx, -1.0) && approx(b.rz, 0.0));
        // yaw 90: facing -X, right = -Z
        let b = Basis::from_yaw_deg(90.0);
        assert!(approx(b.fx, -1.0) && approx(b.fz, 0.0));
        assert!(approx(b.rx, 0.0) && approx(b.rz, -1.0));
        // yaw 180: facing -Z, right = +X
        let b = Basis::from_yaw_deg(180.0);
        assert!(approx(b.fx, 0.0) && approx(b.fz, -1.0));
        assert!(approx(b.rx, 1.0) && approx(b.rz, 0.0));
    }

    #[test]
    fn look_vectors() {
        let r = Ray::from_look(0.0, 64.0, 0.0, 0.0, 0.0);
        // yaw 0, pitch 0 -> looking straight +Z
        assert!(approx(r.dir.x, 0.0) && approx(r.dir.z, 1.0) && approx(r.dir.y, 0.0));
        let r = Ray::from_look(0.0, 64.0, 0.0, 0.0, 90.0);
        // pitch 90 -> straight down
        assert!(approx(r.dir.y, -1.0));
    }

    #[test]
    fn ray_box() {
        let r = Ray::from_look(0.0, 0.0, -5.0, 0.0, 0.0); // +Z ray
        let box1 = Aabb::centered(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let hit = box1.ray_hit(r).expect("must hit");
        assert!(approx(hit, 4.5));
        let miss = Aabb::centered(5.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        assert!(miss.ray_hit(r).is_none());
    }

    #[test]
    fn menu_world_roundtrip() {
        let b = Basis::from_yaw_deg(90.0);
        let anchor = Vec3::new(10.0, 64.0, -5.0);
        let w = menu_to_world(anchor, b, 1.0, 2.0, 3.0);
        // right = -Z, forward = -X, element closer = -forward*z = +X*3
        assert!(approx(w.x, 13.0));
        assert!(approx(w.y, 66.0));
        assert!(approx(w.z, -6.0));
    }
}
