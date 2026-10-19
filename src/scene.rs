use std::{fs, path::Path};
pub type V = [f32; 3];
pub fn add(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn sub(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn mul(a: V, f: f32) -> V {
    a.map(|x| x * f)
}
pub fn dot(a: V, b: V) -> f32 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
pub fn cross(a: V, b: V) -> V {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub fn unit(a: V) -> V {
    mul(a, 1.0 / dot(a, a).sqrt().max(0.00001))
}
pub fn rotate(v: V, axis: usize, degrees: f32) -> V {
    let (s, c) = degrees.to_radians().sin_cos();
    match axis {
        0 => [v[0], c * v[1] - s * v[2], s * v[1] + c * v[2]],
        1 => [c * v[0] + s * v[2], v[1], -s * v[0] + c * v[2]],
        _ => [c * v[0] - s * v[1], s * v[0] + c * v[1], v[2]],
    }
}
pub const COLORS: [[u8; 3]; 10] = [
    [153, 164, 178],
    [62, 146, 199],
    [64, 172, 136],
    [230, 167, 76],
    [191, 91, 95],
    [145, 115, 195],
    [218, 220, 220],
    [69, 80, 96],
    [169, 124, 91],
    [93, 164, 170],
];
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Box,
    Ramp,
    Cylinder,
    Steps,
    Sphere,
    Roof,
}
impl Shape {
    pub const ALL: [Self; 6] = [
        Self::Box,
        Self::Ramp,
        Self::Cylinder,
        Self::Steps,
        Self::Sphere,
        Self::Roof,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Box => "Block",
            Self::Ramp => "Ramp",
            Self::Cylinder => "Cylinder",
            Self::Steps => "Stairs",
            Self::Sphere => "Sphere",
            Self::Roof => "Roof",
        }
    }
    fn key(self) -> &'static str {
        match self {
            Self::Box => "box",
            Self::Ramp => "ramp",
            Self::Cylinder => "cylinder",
            Self::Steps => "steps",
            Self::Sphere => "sphere",
            Self::Roof => "roof",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|x| x.key() == s)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub paint:Vec<crate::paint::Mark>,
    pub grid: Option<crate::object_grid::Grid>,
    pub name: String,
    pub shape: Shape,
    pub at: V,
    pub size: V,
    pub yaw: f32,
    pub tilt: [f32; 2],
    pub color: [u8; 3],
    pub solid: bool,
    pub locked: bool,
}
impl Object {
    pub fn new(name: &str, shape: Shape, at: V, size: V, color: [u8; 3]) -> Self {
        Self {
            paint:vec![],
            grid: None,
            name: name.into(),
            shape,
            at,
            size,
            yaw: 0.0,
            tilt: [0.0; 2],
            color,
            solid: true,
            locked: false,
        }
    }
    pub fn local(&self, p: V) -> V {
        let d = sub(p, self.at);
        rotate(
            rotate(rotate(d, 2, -self.yaw), 1, -self.tilt[1]),
            0,
            -self.tilt[0],
        )
    }
    pub fn world(&self, p: V) -> V {
        add(
            self.at,
            rotate(
                rotate(rotate(p, 0, self.tilt[0]), 1, self.tilt[1]),
                2,
                self.yaw,
            ),
        )
    }
    pub fn rotate_about(&mut self, pivot: V, axis: usize, angle: f32) {
        let cols: [V; 3] = std::array::from_fn(|k| {
            let mut v = [0.0; 3];
            v[k] = 1.0;
            rotate(sub(self.world(v), self.at), axis, angle)
        });
        self.at = add(pivot, rotate(sub(self.at, pivot), axis, angle));
        let y = (-cols[0][2]).clamp(-1.0, 1.0).asin();
        let (x, z) = if y.cos().abs() > 0.0001 {
            (cols[1][2].atan2(cols[2][2]), cols[0][1].atan2(cols[0][0]))
        } else {
            (0.0, (-cols[1][0]).atan2(cols[1][1]))
        };
        self.tilt = [x.to_degrees(), y.to_degrees()];
        self.yaw = z.to_degrees().rem_euclid(360.0);
    }
    pub fn bounds(&self) -> (V, V) {
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for p in self.triangles().into_iter().flatten() {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        if !lo[0].is_finite() {
            for x in [-0.5, 0.5] {
                for y in [-0.5, 0.5] {
                    for z in [0.0, 1.0] {
                        let p = self.world([x * self.size[0], y * self.size[1], z * self.size[2]]);
                        for k in 0..3 {
                            lo[k] = lo[k].min(p[k]);
                            hi[k] = hi[k].max(p[k]);
                        }
                    }
                }
            }
        }
        (lo, hi)
    }
    pub fn triangles(&self) -> Vec<[V; 3]> {
        if let Some(g) = &self.grid {
            return g
                .triangles(self.size)
                .into_iter()
                .map(|t| t.map(|p| self.world(p)))
                .collect();
        }
        let mut t = Vec::new();
        let [w, d, h] = self.size;
        let (x, y) = (w / 2.0, d / 2.0);
        match self.shape {
            Shape::Box => cuboid(&mut t, [-x, -y, 0.0], [x, y, h]),
            Shape::Roof => {
                let p = [
                    [-x, -y, 0.0],
                    [x, -y, 0.0],
                    [x, y, 0.0],
                    [-x, y, 0.0],
                    [0.0, -y, h],
                    [0.0, y, h],
                ];
                for [a, b, c] in [
                    [0, 2, 1],
                    [0, 3, 2],
                    [0, 1, 4],
                    [3, 5, 2],
                    [0, 4, 5],
                    [0, 5, 3],
                    [1, 2, 5],
                    [1, 5, 4],
                ] {
                    t.push([p[a], p[b], p[c]]);
                }
            }
            Shape::Steps => {
                for i in 0..8 {
                    let a = -y + d * i as f32 / 8.0;
                    let b = -y + d * (i + 1) as f32 / 8.0;
                    cuboid(&mut t, [-x, a, 0.0], [x, b, h * (i + 1) as f32 / 8.0]);
                }
            }
            Shape::Ramp => {
                let p = [
                    [-x, -y, 0.0],
                    [x, -y, 0.0],
                    [x, y, 0.0],
                    [-x, y, 0.0],
                    [-x, y, h],
                    [x, y, h],
                ];
                for [a, b, c] in [
                    [0, 2, 1],
                    [0, 3, 2],
                    [0, 1, 5],
                    [0, 5, 4],
                    [3, 4, 5],
                    [3, 5, 2],
                    [0, 4, 3],
                    [1, 2, 5],
                ] {
                    t.push([p[a], p[b], p[c]]);
                }
            }
            Shape::Cylinder => {
                for i in 0..24 {
                    let a = i as f32 * std::f32::consts::TAU / 24.0;
                    let b = (i + 1) as f32 * std::f32::consts::TAU / 24.0;
                    let p = [a.cos() * x, a.sin() * y, 0.0];
                    let q = [b.cos() * x, b.sin() * y, 0.0];
                    let r = [q[0], q[1], h];
                    let s = [p[0], p[1], h];
                    t.extend([
                        [p, q, r],
                        [p, r, s],
                        [[0.0, 0.0, 0.0], q, p],
                        [[0.0, 0.0, h], s, r],
                    ]);
                }
            }
            Shape::Sphere => {
                let p = |i: usize, j: usize| {
                    let a = i as f32 * std::f32::consts::TAU / 24.0;
                    let b = j as f32 * std::f32::consts::PI / 12.0;
                    [
                        a.cos() * b.sin() * x,
                        a.sin() * b.sin() * y,
                        (1.0 - b.cos()) * h / 2.0,
                    ]
                };
                for i in 0..24 {
                    for j in 0..12 {
                        let (a, b, c, d) = (p(i, j), p(i + 1, j), p(i + 1, j + 1), p(i, j + 1));
                        t.extend([[a, c, b], [a, d, c]]);
                    }
                }
            }
        }
        t.into_iter().map(|v| v.map(|p| self.world(p))).collect()
    }
    pub fn valid(&self) -> bool {
        !self.name.is_empty()
            && self.name.len() <= 80
            && !self.name.chars().any(|c| c.is_control() || c == '|')
            && self.at.iter().all(|x| x.is_finite() && x.abs() <= 10000.0)
            && self
                .size
                .iter()
                .all(|x| x.is_finite() && (0.05..=1000.0).contains(x))
            && self
                .tilt
                .iter()
                .all(|x| x.is_finite() && x.abs() <= 36000.0)
            && self.yaw.is_finite()
            && self.yaw.abs() <= 36000.0
    }
    pub fn top(&self, p: V) -> Option<f32> {
        if self.grid.is_some() || self.tilt.iter().any(|x| x.abs() > 0.0001) {
            return self
                .triangles()
                .into_iter()
                .filter_map(|t| {
                    ray_triangle([p[0], p[1], 20000.0], [0.0, 0.0, -1.0], t).map(|d| 20000.0 - d)
                })
                .max_by(f32::total_cmp);
        }
        let q = self.local(p);
        let [w, d, h] = self.size;
        if q[0].abs() > w / 2.0 || q[1].abs() > d / 2.0 {
            return None;
        }
        let t = (q[1] / d + 0.5).clamp(0.0, 1.0);
        Some(
            self.at[2]
                + match self.shape {
                    Shape::Box => h,
                    Shape::Roof => h * (1.0 - (q[0] * 2.0 / w).abs()),
                    Shape::Ramp => t * h,
                    Shape::Steps => (t * 8.0).floor().min(7.0).mul_add(1.0, 1.0) * h / 8.0,
                    Shape::Cylinder => {
                        if (q[0] * 2.0 / w).powi(2) + (q[1] * 2.0 / d).powi(2) > 1.0 {
                            return None;
                        }
                        h
                    }
                    Shape::Sphere => {
                        let r = (q[0] * 2.0 / w).powi(2) + (q[1] * 2.0 / d).powi(2);
                        if r > 1.0 {
                            return None;
                        }
                        h * 0.5 * (1.0 + (1.0 - r).sqrt())
                    }
                },
        )
    }
}
fn cuboid(t: &mut Vec<[V; 3]>, a: V, b: V) {
    let p = [
        [a[0], a[1], a[2]],
        [b[0], a[1], a[2]],
        [b[0], b[1], a[2]],
        [a[0], b[1], a[2]],
        [a[0], a[1], b[2]],
        [b[0], a[1], b[2]],
        [b[0], b[1], b[2]],
        [a[0], b[1], b[2]],
    ];
    for [i, j, k, l] in [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [1, 2, 6, 5],
        [2, 3, 7, 6],
        [3, 0, 4, 7],
    ] {
        t.extend([[p[i], p[j], p[k]], [p[i], p[k], p[l]]]);
    }
}
pub fn ray_triangle(o: V, d: V, t: [V; 3]) -> Option<f32> {
    let e = sub(t[1], t[0]);
    let f = sub(t[2], t[0]);
    let p = cross(d, f);
    let det = dot(e, p);
    if det.abs() < 1e-6 {
        return None;
    }
    let q = sub(o, t[0]);
    let u = dot(q, p) / det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let r = cross(q, e);
    let v = dot(d, r) / det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let at = dot(f, r) / det;
    (at > 0.01).then_some(at)
}
pub fn pick(objects: &[Object], o: V, d: V) -> Option<(usize, f32)> {
    objects
        .iter()
        .enumerate()
        .flat_map(|(i, b)| {
            b.triangles()
                .into_iter()
                .filter_map(move |t| ray_triangle(o, d, t).map(|v| (i, v)))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
}
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub objects: Vec<Object>,
    pub spawn: V,
}
impl Scene {
    pub fn starter() -> Self {
        let mut s = Self {
            objects: vec![],
            spawn: [0.0, -8.0, 0.02],
        };
        let mut put = |n: &str, k, p, z, c| s.objects.push(Object::new(n, k, p, z, COLORS[c]));
        put(
            "Sandbox foundation",
            Shape::Box,
            [0.0, 0.0, -0.5],
            [48.0, 48.0, 0.5],
            0,
        );
        put(
            "North wall",
            Shape::Box,
            [0.0, 23.5, 0.0],
            [48.0, 1.0, 4.0],
            7,
        );
        put(
            "West wall",
            Shape::Box,
            [-23.5, 0.0, 0.0],
            [1.0, 46.0, 4.0],
            7,
        );
        put(
            "East wall",
            Shape::Box,
            [23.5, 0.0, 0.0],
            [1.0, 46.0, 4.0],
            7,
        );
        put(
            "South wall left",
            Shape::Box,
            [-15.0, -23.5, 0.0],
            [18.0, 1.0, 2.0],
            7,
        );
        put(
            "South wall right",
            Shape::Box,
            [15.0, -23.5, 0.0],
            [18.0, 1.0, 2.0],
            7,
        );
        put(
            "Stair test",
            Shape::Steps,
            [-13.0, 6.0, 0.0],
            [4.0, 7.0, 2.4],
            1,
        );
        put(
            "Raised deck",
            Shape::Box,
            [-13.0, 12.0, 0.0],
            [7.0, 5.0, 2.4],
            1,
        );
        put(
            "Ramp test",
            Shape::Ramp,
            [-6.0, 7.0, 0.0],
            [3.0, 8.0, 2.4],
            2,
        );
        put("Low jump", Shape::Box, [4.0, 1.0, 0.0], [3.0, 1.0, 0.4], 3);
        put(
            "Medium jump",
            Shape::Box,
            [4.0, 4.0, 0.0],
            [3.0, 1.0, 0.8],
            3,
        );
        put(
            "Tall obstacle",
            Shape::Box,
            [4.0, 7.0, 0.0],
            [3.0, 1.0, 1.2],
            3,
        );
        put(
            "Round column",
            Shape::Cylinder,
            [13.0, 9.0, 0.0],
            [2.0, 2.0, 4.0],
            5,
        );
        put(
            "Round prop",
            Shape::Sphere,
            [17.0, 9.0, 0.0],
            [2.0, 2.0, 2.0],
            4,
        );
        put(
            "Room back",
            Shape::Box,
            [12.0, -12.0, 0.0],
            [8.0, 0.3, 3.0],
            8,
        );
        put(
            "Room side",
            Shape::Box,
            [16.0, -8.5, 0.0],
            [0.3, 7.0, 3.0],
            8,
        );
        put(
            "Room doorway left",
            Shape::Box,
            [9.0, -5.0, 0.0],
            [2.0, 0.3, 3.0],
            8,
        );
        put(
            "Room doorway right",
            Shape::Box,
            [14.0, -5.0, 0.0],
            [4.0, 0.3, 3.0],
            8,
        );
        put(
            "Door lintel",
            Shape::Box,
            [11.0, -5.0, 2.3],
            [2.0, 0.3, 0.7],
            8,
        );
        s.objects[0].locked = true;
        s
    }
    pub fn ground(&self, p: V, ceiling: f32) -> Option<f32> {
        self.objects
            .iter()
            .filter(|b| b.solid)
            .filter_map(|b| b.top(p))
            .filter(|z| *z <= ceiling)
            .max_by(f32::total_cmp)
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if !self.objects.iter().all(Object::valid)
            || self.objects.len() > 50000
            || !self
                .spawn
                .iter()
                .all(|x| x.is_finite() && x.abs() <= 10000.0)
        {
            return Err("Invalid scene values; existing save preserved".into());
        }
        let mut out = format!(
            "GROVE_SCENE 4\nspawn {} {} {}\n",
            self.spawn[0], self.spawn[1], self.spawn[2]
        );
        for b in &self.objects {
            out += &format!(
                "{}|{}|{} {} {}|{} {} {}|{}|{} {} {}|{}|{}|{} {}|{}|{}\n",
                b.shape.key(),
                b.name,
                b.at[0],
                b.at[1],
                b.at[2],
                b.size[0],
                b.size[1],
                b.size[2],
                b.yaw,
                b.color[0],
                b.color[1],
                b.color[2],
                b.solid as u8,
                b.locked as u8,
                b.tilt[0],
                b.tilt[1],
                b.grid.as_ref().map_or("-".into(), |g| g.encode()),crate::paint::encode(&b.paint)
            );
        }
        if let Some(p) = path.parent() {
            fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, out).map_err(|e| e.to_string())?;
        if path.exists() {
            fs::copy(path, path.with_extension("grove.bak")).map_err(|e| e.to_string())?;
        }
        fs::rename(tmp, path).map_err(|e| e.to_string())
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        if text.len() > 32_000_000 {
            return Err("Scene is too large".into());
        }
        let mut lines = text.lines();
        let version = lines.next();
        if !matches!(
            version,
            Some("GROVE_SCENE 1" | "GROVE_SCENE 2" | "GROVE_SCENE 3" | "GROVE_SCENE 4")
        ) {
            return Err("Unrecognized scene version".into());
        }
        let sp = lines
            .next()
            .ok_or("Missing spawn")?
            .strip_prefix("spawn ")
            .ok_or("Missing spawn")?;
        let spawn = vec3(sp)?;
        if spawn.iter().any(|x| !x.is_finite() || x.abs() > 10000.0) {
            return Err("Invalid spawn".into());
        }
        let mut objects = Vec::new();
        for (n, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let f: Vec<_> = line.split('|').collect();
            let parse = || -> Result<Object, String> {
                if f.len()
                    != if version == Some("GROVE_SCENE 1") {
                        8
                    } else if version == Some("GROVE_SCENE 2") {
                        9
                    } else if version==Some("GROVE_SCENE 3") {
                        10
                    }else{11
                    }
                {
                    return Err("Invalid fields".into());
                }
                let color: Vec<u8> = f[5]
                    .split_whitespace()
                    .map(|x| x.parse().map_err(|_| "Invalid color".to_string()))
                    .collect::<Result<_, _>>()?;
                if color.len() != 3 || !matches!(f[6], "0" | "1") || !matches!(f[7], "0" | "1") {
                    return Err("Invalid color or flags".into());
                }
                let b = Object {
                    paint:if f.len()==11{crate::paint::decode(f[10])?}else{vec![]},
                    grid: if f.len() >= 10 && f[9] != "-" {
                        Some(crate::object_grid::Grid::decode(f[9])?)
                    } else {
                        None
                    },
                    name: f[1].into(),
                    shape: Shape::parse(f[0]).ok_or("Invalid shape")?,
                    at: vec3(f[2])?,
                    size: vec3(f[3])?,
                    yaw: f[4].parse().map_err(|_| "Invalid rotation")?,
                    tilt: if f.len() >= 9 {
                        let v: Vec<f32> = f[8]
                            .split_whitespace()
                            .map(|x| x.parse().map_err(|_| "Invalid tilt"))
                            .collect::<Result<_, _>>()?;
                        v.try_into().map_err(|_| "Invalid tilt")?
                    } else {
                        [0.0; 2]
                    },
                    color: [color[0], color[1], color[2]],
                    solid: f[6] == "1",
                    locked: f[7] == "1",
                };
                if !b.valid() {
                    return Err("Invalid object".into());
                }
                Ok(b)
            };
            objects.push(parse().map_err(|e| format!("Line {}: {e}", n + 3))?);
            if objects.len() > 50000 {
                return Err("Too many objects".into());
            }
        }
        Ok(Self { objects, spawn })
    }
}
fn vec3(s: &str) -> Result<V, String> {
    let v: Vec<f32> = s
        .split_whitespace()
        .map(|x| x.parse().map_err(|_| "Invalid number".to_string()))
        .collect::<Result<_, _>>()?;
    v.try_into().map_err(|_| "Expected three numbers".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sloped_surface_and_rotated_selection_agree() {
        let mut o = Object::new(
            "ramp",
            Shape::Ramp,
            [3.0, 4.0, 1.0],
            [4.0, 8.0, 2.0],
            COLORS[0],
        );
        o.yaw = 90.0;
        let p = o.world([0.0, 2.0, 0.0]);
        let top = o.top(p).unwrap();
        assert!((top - 2.5).abs() < 1e-5);
        let (_, t) = pick(&[o], [p[0], p[1], 10.0], [0.0, 0.0, -1.0]).unwrap();
        assert!((10.0 - t - top).abs() < 1e-4);
    }
    #[test]
    fn scene_roundtrip_and_invalid_save_preserve_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("grove-scene-test-{}.grove", std::process::id()));
        let s = Scene::starter();
        s.save(&path).unwrap();
        assert_eq!(Scene::load(&path).unwrap(), s);
        let before = fs::read(&path).unwrap();
        let mut bad = s;
        bad.objects[0].size[0] = f32::NAN;
        assert!(bad.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn carved_tilted_object_saves_and_legacy_worlds_still_load() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("grove-grid-migration-{}.grove", std::process::id()));
        fs::write(
            &path,
            "GROVE_SCENE 1\nspawn 0 0 0\nbox|Legacy wall|0 0 0|4 0.25 3|0|62 146 199|1|0\n",
        )
        .unwrap();
        let mut s = Scene::load(&path).unwrap();
        let b = &mut s.objects[0];
        let mut g = crate::object_grid::Grid::from_object(b, 16);
        g.edit([6, 0, 0], [9, 0, 8], 1, true, false);
        b.grid = Some(g);
        b.tilt = [13.0, 24.0];
        b.yaw = 40.0;
        let before = b.triangles();
        s.save(&path).unwrap();
        let loaded = Scene::load(&path).unwrap();
        assert_eq!(loaded, s);
        assert_eq!(loaded.objects[0].triangles(), before);
        assert!(fs::read_to_string(path.with_extension("grove.bak"))
            .unwrap()
            .starts_with("GROVE_SCENE 1"));
        fs::remove_file(&path).unwrap();
        fs::remove_file(path.with_extension("grove.bak")).unwrap();
    }
}
