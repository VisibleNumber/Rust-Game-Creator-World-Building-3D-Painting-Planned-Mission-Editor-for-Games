use crate::scene::*;
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub dims: [usize; 3],
    pub cells: Vec<bool>,
}
impl Grid {
    pub fn index(&self, p: [usize; 3]) -> usize {
        (p[2] * self.dims[1] + p[1]) * self.dims[0] + p[0]
    }
    pub fn get(&self, p: [i32; 3]) -> bool {
        if (0..3).any(|k| p[k] < 0 || p[k] >= self.dims[k] as i32) {
            return false;
        }
        self.cells[self.index(p.map(|x| x as usize))]
    }
    pub fn sample(&self, p: V, size: V) -> bool {
        let q = [p[0] + size[0] / 2.0, p[1] + size[1] / 2.0, p[2]];
        self.get(std::array::from_fn(|k| {
            (q[k] / size[k] * self.dims[k] as f32).floor() as i32
        }))
    }
    pub fn from_object(b: &Object, res: usize) -> Self {
        let max = b.size.into_iter().fold(0.0, f32::max);
        let step = max / res as f32;
        let dims = b.size.map(|s| (s / step).ceil().clamp(1.0, 32.0) as usize);
        let mut g = Self {
            dims,
            cells: vec![false; dims.into_iter().product()],
        };
        for z in 0..dims[2] {
            for y in 0..dims[1] {
                for x in 0..dims[0] {
                    let p = [x, y, z];
                    let q: V = std::array::from_fn(|k| {
                        (p[k] as f32 + 0.5) / dims[k] as f32 * b.size[k]
                            - if k < 2 { b.size[k] / 2.0 } else { 0.0 }
                    });
                    let filled = if let Some(old) = &b.grid {
                        old.sample(q, b.size)
                    } else {
                        match b.shape {
                            Shape::Box => true,
                            Shape::Ramp => q[2] <= b.size[2] * (q[1] / b.size[1] + 0.5),
                            Shape::Roof => {
                                q[2] <= b.size[2] * (1.0 - (2.0 * q[0] / b.size[0]).abs())
                            }
                            Shape::Steps => {
                                q[2] <= ((q[1] / b.size[1] + 0.5) * 8.0)
                                    .floor()
                                    .min(7.0)
                                    .mul_add(1.0, 1.0)
                                    * b.size[2]
                                    / 8.0
                            }
                            Shape::Cylinder => {
                                (q[0] * 2.0 / b.size[0]).powi(2) + (q[1] * 2.0 / b.size[1]).powi(2)
                                    <= 1.0
                            }
                            Shape::Sphere => {
                                (q[0] * 2.0 / b.size[0]).powi(2)
                                    + (q[1] * 2.0 / b.size[1]).powi(2)
                                    + (q[2] * 2.0 / b.size[2] - 1.0).powi(2)
                                    <= 1.0
                            }
                        }
                    };
                    let i = g.index(p);
                    g.cells[i] = filled;
                }
            }
        }
        g
    }
    pub fn edit(&mut self, a: [usize; 3], b: [usize; 3], axis: usize, through: bool, fill: bool) {
        for z in 0..self.dims[2] {
            for y in 0..self.dims[1] {
                for x in 0..self.dims[0] {
                    let p = [x, y, z];
                    if (0..3).all(|k| {
                        (k == axis && through) || (p[k] >= a[k].min(b[k]) && p[k] <= a[k].max(b[k]))
                    }) {
                        let i = self.index(p);
                        self.cells[i] = fill;
                    }
                }
            }
        }
    }
    pub fn triangles(&self, size: V) -> Vec<[V; 3]> {
        let mut out = Vec::new();
        for z in 0..self.dims[2] {
            for y in 0..self.dims[1] {
                for x in 0..self.dims[0] {
                    let p = [x as i32, y as i32, z as i32];
                    if !self.get(p) {
                        continue;
                    }
                    let a: V = std::array::from_fn(|k| {
                        p[k] as f32 / self.dims[k] as f32 * size[k]
                            - if k < 2 { size[k] / 2.0 } else { 0.0 }
                    });
                    let b: V = std::array::from_fn(|k| {
                        (p[k] + 1) as f32 / self.dims[k] as f32 * size[k]
                            - if k < 2 { size[k] / 2.0 } else { 0.0 }
                    });
                    let v = [
                        [a[0], a[1], a[2]],
                        [b[0], a[1], a[2]],
                        [b[0], b[1], a[2]],
                        [a[0], b[1], a[2]],
                        [a[0], a[1], b[2]],
                        [b[0], a[1], b[2]],
                        [b[0], b[1], b[2]],
                        [a[0], b[1], b[2]],
                    ];
                    for (k, sign, face) in [
                        (2, -1, [0, 3, 2, 1]),
                        (2, 1, [4, 5, 6, 7]),
                        (1, -1, [0, 1, 5, 4]),
                        (0, 1, [1, 2, 6, 5]),
                        (1, 1, [2, 3, 7, 6]),
                        (0, -1, [3, 0, 4, 7]),
                    ] {
                        let mut n = p;
                        n[k] += sign;
                        if self.get(n) {
                            continue;
                        }
                        let [i, j, k, l] = face;
                        out.extend([[v[i], v[j], v[k]], [v[i], v[k], v[l]]]);
                    }
                }
            }
        }
        out
    }
    pub fn encode(&self) -> String {
        let mut s = format!("{} {} {}:", self.dims[0], self.dims[1], self.dims[2]);
        for chunk in self.cells.chunks(4) {
            let mut n = 0u8;
            for (i, &v) in chunk.iter().enumerate() {
                if v {
                    n |= 1 << i;
                }
            }
            s.push(char::from_digit(n as u32, 16).unwrap());
        }
        s
    }
    pub fn decode(s: &str) -> Result<Self, String> {
        let (a, b) = s.split_once(':').ok_or("Invalid object grid")?;
        let d: Vec<usize> = a
            .split_whitespace()
            .map(|x| x.parse().map_err(|_| "Invalid grid size"))
            .collect::<Result<_, _>>()?;
        let dims: [usize; 3] = d.try_into().map_err(|_| "Invalid grid dimensions")?;
        if dims.iter().any(|x| !(1..=64).contains(x)) {
            return Err("Grid dimensions must be 1 to 64".into());
        }
        let count: usize = dims.into_iter().product();
        if b.len() != (count + 3) / 4 {
            return Err("Invalid grid data length".into());
        }
        let mut cells = Vec::new();
        for c in b.chars() {
            let n = c.to_digit(16).ok_or("Invalid grid data")?;
            for k in 0..4 {
                cells.push(n & (1 << k) != 0);
            }
        }
        cells.truncate(count);
        Ok(Self { dims, cells })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cuts_fill_and_hex_roundtrip_are_lossless() {
        let b = Object::new("block", Shape::Box, [0.0; 3], [4.0, 1.0, 4.0], COLORS[0]);
        let mut g = Grid::from_object(&b, 16);
        let full = g.clone();
        g.edit([6, 0, 0], [9, 0, 8], 1, true, false);
        assert!(!g.get([7, 1, 4]));
        assert!(g.get([2, 1, 4]));
        assert_eq!(Grid::decode(&g.encode()).unwrap(), g);
        g.edit([6, 0, 0], [9, 0, 8], 1, true, true);
        assert_eq!(g, full);
        assert!(Grid::decode("999 1 1:f").is_err());
    }
    #[test]
    fn internal_cell_faces_are_not_emitted() {
        let g = Grid {
            dims: [2, 1, 1],
            cells: vec![true, true],
        };
        assert_eq!(g.triangles([2.0, 1.0, 1.0]).len(), 20);
    }
}
