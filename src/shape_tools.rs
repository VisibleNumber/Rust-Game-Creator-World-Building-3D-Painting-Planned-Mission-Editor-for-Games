use crate::{object_grid::Grid, scene::*};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Brush {
    Rectangle,
    Ellipse,
    Triangle,
    Arch,
    Diamond,
    Ball,
    Freehand,
}
impl Brush {
    pub const ALL: [Self; 7] = [
        Self::Rectangle,
        Self::Ellipse,
        Self::Triangle,
        Self::Arch,
        Self::Diamond,
        Self::Ball,
        Self::Freehand,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Rectangle => "Box",
            Self::Ellipse => "Circle",
            Self::Triangle => "Triangle",
            Self::Arch => "Arch",
            Self::Diamond => "Diamond",
            Self::Ball => "Ball",
            Self::Freehand => "Draw",
        }
    }
}
#[derive(Clone)]
pub struct Edit {
    pub a: [usize; 3],
    pub b: [usize; 3],
    pub axis: usize,
    pub side: i32,
    pub through: bool,
    pub depth: usize,
    pub fill: bool,
    pub brush: Brush,
    pub mirror: u8,
    pub radius: usize,
    pub path: Vec<[usize; 3]>,
}
impl Grid {
    pub fn paint(&mut self, e: &Edit) -> usize {
        if e.brush == Brush::Freehand {
            let mut changed = 0;
            for &point in &e.path {
                let mut stroke = Edit {
                    a: point,
                    b: point,
                    axis: e.axis,
                    side: e.side,
                    through: e.through,
                    depth: e.depth,
                    fill: e.fill,
                    brush: Brush::Ellipse,
                    mirror: e.mirror,
                    radius: e.radius,
                    path: vec![],
                };
                stroke.a = point;
                stroke.b = point;
                for k in 0..3 {
                    if k != e.axis {
                        stroke.a[k] = point[k].saturating_sub(e.radius);
                        stroke.b[k] = (point[k] + e.radius).min(self.dims[k] - 1);
                    }
                }
                changed += self.paint(&stroke);
            }
            return changed;
        }
        let mut lo: [usize; 3] = std::array::from_fn(|k| e.a[k].min(e.b[k]).min(self.dims[k] - 1));
        let mut hi: [usize; 3] = std::array::from_fn(|k| e.a[k].max(e.b[k]).min(self.dims[k] - 1));
        if e.through {
            lo[e.axis] = 0;
            hi[e.axis] = self.dims[e.axis] - 1;
        } else if e.side < 0 {
            lo[e.axis] = e.a[e.axis].min(self.dims[e.axis] - 1);
            hi[e.axis] = (lo[e.axis] + e.depth.max(1) - 1).min(self.dims[e.axis] - 1);
        } else {
            hi[e.axis] = e.a[e.axis].min(self.dims[e.axis] - 1);
            lo[e.axis] = (hi[e.axis] + 1).saturating_sub(e.depth.max(1));
        }
        let axes: Vec<usize> = (0..3).filter(|&k| k != e.axis).collect();
        let (u, v) = if axes.contains(&2) {
            (axes.iter().copied().find(|&k| k != 2).unwrap(), 2)
        } else {
            (axes[0], axes[1])
        };
        let mut changed = 0;
        for z in lo[2]..=hi[2] {
            for y in lo[1]..=hi[1] {
                for x in lo[0]..=hi[0] {
                    let p = [x, y, z];
                    let q: V = std::array::from_fn(|k| {
                        ((p[k] - lo[k]) as f32 + 0.5) / (hi[k] - lo[k] + 1) as f32 * 2.0 - 1.0
                    });
                    let inside = match e.brush {
                        Brush::Rectangle => true,
                        Brush::Ellipse => q[u] * q[u] + q[v] * q[v] <= 1.0,
                        Brush::Triangle => q[u].abs() <= (1.0 - q[v]) / 2.0,
                        Brush::Arch => q[v] <= 0.0 || q[u] * q[u] + q[v] * q[v] <= 1.0,
                        Brush::Diamond => q[u].abs() + q[v].abs() <= 1.0,
                        Brush::Ball => dot(q, q) <= 1.0,
                        Brush::Freehand => unreachable!(),
                    };
                    if !inside {
                        continue;
                    }
                    for mask in 0..8u8 {
                        if mask & !e.mirror != 0 {
                            continue;
                        }
                        let mirrored = std::array::from_fn(|k| {
                            if mask & (1 << k) != 0 {
                                self.dims[k] - 1 - p[k]
                            } else {
                                p[k]
                            }
                        });
                        let i = self.index(mirrored);
                        if self.cells[i] != e.fill {
                            self.cells[i] = e.fill;
                            changed += 1;
                        }
                    }
                }
            }
        }
        changed
    }
    pub fn grow(&mut self, axis: usize, side: i32, n: usize) -> Result<(), String> {
        if axis > 2 || n == 0 || self.dims[axis] + n > 64 {
            return Err("Object canvas limit reached (64 cells per axis). Use a coarser grid or another piece.".into());
        }
        let old = self.clone();
        self.dims[axis] += n;
        self.cells = vec![false; self.dims.into_iter().product()];
        for z in 0..old.dims[2] {
            for y in 0..old.dims[1] {
                for x in 0..old.dims[0] {
                    let p = [x, y, z];
                    let mut q = p;
                    if side < 0 {
                        q[axis] += n;
                    }
                    let i = self.index(q);
                    self.cells[i] = old.get(p.map(|v| v as i32));
                }
            }
        }
        Ok(())
    }
    pub fn hollow(&mut self, thickness: usize) {
        let old = self.clone();
        for z in 0..self.dims[2] {
            for y in 0..self.dims[1] {
                for x in 0..self.dims[0] {
                    let p = [x, y, z];
                    let q = p.map(|v| v as i32);
                    if !old.get(q) {
                        continue;
                    }
                    let mut interior = true;
                    for k in 0..3 {
                        for step in 1..=thickness.max(1) {
                            for sign in [-1, 1] {
                                let mut n = q;
                                n[k] += step as i32 * sign;
                                if !old.get(n) {
                                    interior = false;
                                }
                            }
                        }
                    }
                    if interior {
                        let i = self.index(p);
                        self.cells[i] = false;
                    }
                }
            }
        }
    }
    pub fn round_edges(&mut self) {
        let old = self.clone();
        let directions = old.dims.iter().filter(|&&d| d > 1).count() * 2;
        if directions == 0 {
            return;
        }
        for z in 0..self.dims[2] {
            for y in 0..self.dims[1] {
                for x in 0..self.dims[0] {
                    let p = [x, y, z];
                    let q = p.map(|v| v as i32);
                    let mut count = 0;
                    for k in 0..3 {
                        if self.dims[k] <= 1 {
                            continue;
                        }
                        for sign in [-1, 1] {
                            let mut n = q;
                            n[k] += sign;
                            count += old.get(n) as usize;
                        }
                    }
                    let i = self.index(p);
                    if old.cells[i] && count <= directions / 2 {
                        self.cells[i] = false;
                    } else if !old.cells[i] && count >= directions - 1 {
                        self.cells[i] = true;
                    }
                }
            }
        }
    }
}
impl Object {
    pub fn contains(&self, world: V) -> bool {
        let q = self.local(world);
        if let Some(g) = &self.grid {
            return g.sample(q, self.size);
        }
        let [w, d, h] = self.size;
        if q[0].abs() > w / 2.0 || q[1].abs() > d / 2.0 || q[2] < 0.0 || q[2] > h {
            return false;
        }
        match self.shape {
            Shape::Box => true,
            Shape::Ramp => q[2] <= h * (q[1] / d + 0.5),
            Shape::Roof => q[2] <= h * (1.0 - (2.0 * q[0] / w).abs()),
            Shape::Steps => {
                q[2] <= ((q[1] / d + 0.5) * 8.0).floor().min(7.0).mul_add(1.0, 1.0) * h / 8.0
            }
            Shape::Cylinder => (2.0 * q[0] / w).powi(2) + (2.0 * q[1] / d).powi(2) <= 1.0,
            Shape::Sphere => {
                (2.0 * q[0] / w).powi(2) + (2.0 * q[1] / d).powi(2) + (2.0 * q[2] / h - 1.0).powi(2)
                    <= 1.0
            }
        }
    }
    pub fn grow_axis(&mut self, axis: usize, side: i32, n: usize) -> Result<(), String> {
        let grid = self.grid.as_mut().ok_or("Enter object editing first")?;
        let step = self.size[axis] / grid.dims[axis] as f32;
        grid.grow(axis, side, n)?;
        let mut offset = [0.0; 3];
        offset[axis] = if axis < 2 {
            side as f32 * n as f32 * step / 2.0
        } else if side < 0 {
            -(n as f32) * step
        } else {
            0.0
        };
        self.at = self.world(offset);
        self.size[axis] += step * n as f32;
        if !self.valid() {
            return Err("Expanded object exceeds the world or size limits".into());
        }
        Ok(())
    }
    pub fn extrude(&mut self, e: &Edit) -> Result<(), String> {
        let n = e.depth.max(1);
        let old_dim = self.grid.as_ref().unwrap().dims[e.axis];
        let source = e.a[e.axis];
        let mut low = if e.side < 0 {
            n.saturating_sub(source)
        } else {
            0
        };
        let mut high = if e.side > 0 {
            (source + n + 1).saturating_sub(old_dim)
        } else {
            0
        };
        if e.mirror & (1 << e.axis) != 0 {
            low = low.max(high);
            high = low;
        }
        if low > 0 {
            self.grow_axis(e.axis, -1, low)?;
        }
        if high > 0 {
            self.grow_axis(e.axis, 1, high)?;
        }
        let mut spec = e.clone();
        spec.through = false;
        spec.fill = true;
        spec.a[e.axis] = if e.side > 0 {
            source + low + n
        } else {
            source + low - n
        };
        spec.b[e.axis] = spec.a[e.axis];
        if spec.brush == Brush::Freehand {
            for p in &mut spec.path {
                p[e.axis] = spec.a[e.axis];
            }
        }
        self.grid.as_mut().unwrap().paint(&spec);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn object() -> Object {
        let mut b = Object::new("test", Shape::Box, [2.0, 3.0, 4.0], [4.0; 3], COLORS[0]);
        b.grid = Some(Grid::from_object(&b, 8));
        b
    }
    fn edit(brush: Brush) -> Edit {
        Edit {
            a: [1, 0, 1],
            b: [6, 0, 6],
            axis: 1,
            side: -1,
            through: true,
            depth: 2,
            fill: false,
            brush,
            mirror: 0,
            radius: 1,
            path: vec![[2, 0, 2], [3, 0, 3], [4, 0, 4]],
        }
    }
    #[test]
    fn brushes_make_different_shapes_and_preserve_outside_material() {
        for brush in Brush::ALL {
            let mut b = object();
            let g = b.grid.as_mut().unwrap();
            let changed = g.paint(&edit(brush));
            assert!(changed > 0, "{:?}", brush);
            assert!(g.get([7, 7, 7]));
            assert!(g.cells.iter().any(|v| *v));
        }
    }
    #[test]
    fn depth_and_mirror_are_local_and_bounded() {
        let mut b = object();
        let g = b.grid.as_mut().unwrap();
        let mut e = edit(Brush::Rectangle);
        e.a = [1, 0, 1];
        e.b = [2, 0, 2];
        e.through = false;
        e.depth = 2;
        e.mirror = 1;
        g.paint(&e);
        assert!(!g.get([1, 0, 1]));
        assert!(!g.get([6, 1, 1]));
        assert!(g.get([1, 2, 1]));
    }
    #[test]
    fn growing_rotated_canvas_keeps_existing_cells_in_world_position() {
        let mut b = object();
        b.yaw = 35.0;
        b.tilt = [15.0, 20.0];
        let original = b.clone();
        for axis in 0..3 {
            for side in [-1, 1] {
                b.grow_axis(axis, side, 2).unwrap();
            }
        }
        for q in [[-1.0, -1.0, 0.5], [1.0, 1.0, 3.5], [0.0, 0.0, 2.0]] {
            assert!(b.contains(original.world(q)));
        }
        assert!(!b.contains(b.world([-2.8, -2.8, 0.1])));
    }
    #[test]
    fn pull_out_adds_material_connected_to_source_face() {
        let mut b = object();
        let old = b.clone();
        let mut e = edit(Brush::Rectangle);
        e.a = [2, 0, 2];
        e.b = [5, 0, 5];
        e.through = false;
        e.depth = 2;
        e.fill = true;
        b.extrude(&e).unwrap();
        assert!(b.contains(old.world([0.0, -2.5, 2.0])));
        assert!(b.contains(old.world([0.0, -1.8, 2.0])));
        assert!(!b.contains(old.world([1.8, -2.5, 2.0])));
        assert_eq!(
            Grid::decode(&b.grid.as_ref().unwrap().encode()).unwrap(),
            *b.grid.as_ref().unwrap()
        );
    }
    #[test]
    fn hollow_preserves_shell_and_removes_center() {
        let mut b = object();
        let g = b.grid.as_mut().unwrap();
        g.hollow(1);
        assert!(g.get([0, 3, 3]));
        assert!(!g.get([3, 3, 3]));
    }
}
