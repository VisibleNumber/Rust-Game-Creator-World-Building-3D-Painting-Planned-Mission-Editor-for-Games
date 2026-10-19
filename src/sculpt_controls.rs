struct Carve {
    before: Scene,
    index: usize,
    axis: usize,
    face: f32,
    side: i32,
    start: [usize; 3],
    end: [usize; 3],
    path: Vec<[usize; 3]>,
}
impl App {
    fn sculpt_action(&mut self, a: Action) {
        match a {
            Action::Sculpt => {
                if self.sculpting {
                    self.sculpting = false;
                    return;
                }
                let Some(&i) = self.selected.last() else {
                    self.status = "Select an object, then press E to edit its shape".into();
                    return;
                };
                if self.scene.objects[i].locked {
                    self.status = "Unlock the selected object first".into();
                    return;
                }
                if self.scene.objects[i].grid.is_none() {
                    let before = self.scene.clone();
                    self.scene.objects[i].grid = Some(crate::object_grid::Grid::from_object(
                        &self.scene.objects[i],
                        self.sculpt_resolution.min(32),
                    ));
                    self.checkpoint(before);
                }
                self.sculpting = true;
                self.tool = Tool::Select;
                self.sculpt_resolution = *self.scene.objects[i]
                    .grid
                    .as_ref()
                    .unwrap()
                    .dims
                    .iter()
                    .max()
                    .unwrap();
                self.status =
                    "Choose Cut, Add or Pull out, then a brush. Drag on the object to shape it."
                        .into();
            }
            Action::ObjectMode(mode) => {
                self.fill = mode != 0;
                self.extrude = mode == 2;
                self.status=match mode{0=>"CUT: drag to remove the brush shape",1=>"ADD: drag to add the brush shape. Expand space makes room outside the object.",_=>"PULL OUT: drag a region, then release to extend that face outward"}.into();
            }
            Action::Brush(brush) => {
                self.brush = brush;
                self.status = format!(
                    "{} brush: drag on a face. The colored shape previews your edit.",
                    brush.name()
                );
            }
            Action::Depth(n) => self.depth = (self.depth as i32 + n).clamp(1, 16) as usize,
            Action::Radius(n) => {
                self.brush_radius = (self.brush_radius as i32 + n).clamp(0, 8) as usize
            }
            Action::Mirror(axis) => self.mirror ^= 1 << axis,
            Action::Fill => {
                self.fill = !self.fill;
                self.extrude = false;
            }
            Action::Through => self.through = !self.through,
            Action::SculptGrid => {
                self.sculpt_resolution = match self.sculpt_resolution {
                    8 => 16,
                    16 => 32,
                    _ => 8,
                };
                if self.sculpting {
                    if let Some(&i) = self.selected.last() {
                        let before = self.scene.clone();
                        self.scene.objects[i].grid = Some(crate::object_grid::Grid::from_object(
                            &self.scene.objects[i],
                            self.sculpt_resolution,
                        ));
                        self.checkpoint(before);
                    }
                }
                self.status =
                    "Object grid changed. Coarser grids simplify detail; Ctrl+Z restores it."
                        .into();
            }
            Action::Hollow | Action::RoundEdges | Action::Pad => {
                if !self.sculpting {
                    self.sculpt_action(Action::Sculpt);
                    if !self.sculpting {
                        return;
                    }
                }
                let Some(&i) = self.selected.last() else {
                    return;
                };
                let before = self.scene.clone();
                let mut b = self.scene.objects[i].clone();
                match a {
                    Action::Hollow => {
                        b.grid.as_mut().unwrap().hollow(self.depth);
                        self.status = format!(
                            "Hollowed object with {}-cell walls. Ctrl+Z restores it.",
                            self.depth
                        );
                    }
                    Action::RoundEdges => {
                        b.grid.as_mut().unwrap().round_edges();
                        self.status =
                            "Edges softened by one grid step. Repeat for more; Ctrl+Z restores."
                                .into();
                    }
                    Action::Pad => {
                        for axis in 0..3 {
                            for sign in [-1, 1] {
                                if let Err(e) = b.grow_axis(axis, sign, 2) {
                                    self.status = e;
                                    return;
                                }
                            }
                        }
                        self.status="Expanded empty editing space. Existing geometry stayed in place; use Add to build out.".into();
                    }
                    _ => {}
                }
                self.scene.objects[i] = b;
                self.checkpoint(before);
            }
            _ => {}
        }
    }
    fn edit_spec(&self, c: &Carve) -> Edit {
        Edit {
            a: c.start,
            b: c.end,
            axis: c.axis,
            side: c.side,
            through: self.through,
            depth: self.depth,
            fill: self.fill,
            brush: self.brush,
            mirror: self.mirror,
            radius: self.brush_radius,
            path: c.path.clone(),
        }
    }
    fn begin_cut(&mut self, d: V) {
        let Some(&i) = self.selected.last() else {
            return;
        };
        let b = &self.scene.objects[i];
        let o = b.local(self.cam.at);
        let dir = sub(b.local(add(self.cam.at, d)), o);
        let lo = [-b.size[0] / 2.0, -b.size[1] / 2.0, 0.0];
        let hi = [b.size[0] / 2.0, b.size[1] / 2.0, b.size[2]];
        let (mut near, mut far) = (f32::NEG_INFINITY, f32::INFINITY);
        let mut axis = 0;
        let mut face = 0.0;
        for k in 0..3 {
            if dir[k].abs() < 0.00001 {
                if o[k] < lo[k] || o[k] > hi[k] {
                    return;
                }
                continue;
            }
            let a = (lo[k] - o[k]) / dir[k];
            let z = (hi[k] - o[k]) / dir[k];
            if a.min(z) > near {
                near = a.min(z);
                axis = k;
                face = if a < z { lo[k] } else { hi[k] };
            }
            far = far.min(a.max(z));
            if near > far {
                return;
            }
        }
        if far < 0.0 {
            return;
        }
        let mut q = add(o, mul(dir, near.max(0.0)));
        let mut side = if dir[axis] < 0.0 { 1 } else { -1 };
        if self.extrude {
            let hit = b
                .triangles()
                .into_iter()
                .filter_map(|t| ray_triangle(self.cam.at, d, t).map(|distance| (distance, t)))
                .min_by(|a, b| a.0.total_cmp(&b.0));
            let Some((distance, tri)) = hit else {
                self.status = "Pull out needs a solid face. Use Add to fill empty space.".into();
                return;
            };
            let points = tri.map(|p| b.local(p));
            let normal = cross(sub(points[1], points[0]), sub(points[2], points[0]));
            axis = (0..3)
                .max_by(|&a, &z| normal[a].abs().total_cmp(&normal[z].abs()))
                .unwrap();
            side = if dir[axis] < 0.0 { 1 } else { -1 };
            q = b.local(add(self.cam.at, mul(d, distance)));
            face = q[axis];
            q[axis] -= side as f32 * 0.0001;
        }
        let Some(g) = &b.grid else {
            return;
        };
        let cell = grid_cell(q, b.size, g.dims);
        self.carve = Some(Carve {
            before: self.scene.clone(),
            index: i,
            axis,
            face,
            side,
            start: cell,
            end: cell,
            path: vec![cell],
        });
        self.update_cut(self.pointer);
        unsafe {
            SetCapture(GetForegroundWindow());
        }
    }
    fn update_cut(&mut self, p: [f32; 2]) {
        let Some(c) = &self.carve else {
            return;
        };
        let b = &self.scene.objects[c.index];
        let o = b.local(self.cam.at);
        let dir = sub(
            b.local(add(
                self.cam.at,
                self.ray(p, self.viewport[0], self.viewport[1]),
            )),
            o,
        );
        if dir[c.axis].abs() < 0.00001 {
            return;
        }
        let q = add(o, mul(dir, (c.face - o[c.axis]) / dir[c.axis]));
        let mut end = grid_cell(q, b.size, b.grid.as_ref().unwrap().dims);
        end[c.axis] = c.start[c.axis];
        let c = self.carve.as_mut().unwrap();
        c.end = end;
        if c.path.last() != Some(&end) && c.path.len() < 4096 {
            let previous = *c.path.last().unwrap();
            let steps = (0..3)
                .map(|k| previous[k].abs_diff(end[k]))
                .max()
                .unwrap()
                .max(1);
            for step in 1..=steps {
                let cell = std::array::from_fn(|k| {
                    (previous[k] as f32
                        + (end[k] as f32 - previous[k] as f32) * step as f32 / steps as f32)
                        .round() as usize
                });
                if c.path.len() < 4096 {
                    c.path.push(cell);
                }
            }
        }
        let c = self.carve.as_ref().unwrap();
        let mut preview = self.scene.objects[c.index].clone();
        let mut spec = self.edit_spec(c);
        spec.fill = true;
        preview.grid.as_mut().unwrap().cells.fill(false);
        if self.extrude {
            if let Err(e) = preview.extrude(&spec) {
                self.status = e;
                self.cut_preview = None;
                return;
            }
        } else {
            preview.grid.as_mut().unwrap().paint(&spec);
        }
        preview.color = if self.fill {
            [80, 220, 160]
        } else {
            [235, 85, 90]
        };
        trim_preview(&mut preview);
        self.cut_preview = Some(preview);
    }
    fn end_cut(&mut self, cancel: bool) {
        if let Some(c) = self.carve.take() {
            if !cancel {
                let spec = self.edit_spec(&c);
                let mut b = self.scene.objects[c.index].clone();
                let result = if self.extrude {
                    b.extrude(&spec)
                } else {
                    b.grid.as_mut().unwrap().paint(&spec);
                    Ok(())
                };
                match result {
                    Ok(()) => {
                        self.scene.objects[c.index] = b;
                        self.checkpoint(c.before);
                        self.status = if self.extrude {
                            "Face extended. All changes are one undo step."
                        } else if self.fill {
                            "Shape added. Ctrl+Z undoes."
                        } else {
                            "Shape cut. Add restores material; Ctrl+Z undoes."
                        }
                        .into();
                    }
                    Err(e) => self.status = e,
                }
            }
            self.cut_preview = None;
            unsafe {
                ReleaseCapture();
            }
        }
    }
}
fn grid_cell(q: V, size: V, dims: [usize; 3]) -> [usize; 3] {
    std::array::from_fn(|k| {
        let origin = if k < 2 { size[k] / 2.0 } else { 0.0 };
        (((q[k] + origin) / size[k] * dims[k] as f32).floor() as i32).clamp(0, dims[k] as i32 - 1)
            as usize
    })
}
fn trim_preview(b: &mut Object) {
    let g = b.grid.as_ref().unwrap();
    let mut lo = g.dims;
    let mut hi = [0; 3];
    let mut found = false;
    for z in 0..g.dims[2] {
        for y in 0..g.dims[1] {
            for x in 0..g.dims[0] {
                let p = [x, y, z];
                if !g.get(p.map(|v| v as i32)) {
                    continue;
                }
                found = true;
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
        }
    }
    if !found {
        return;
    }
    let dims = std::array::from_fn(|k| hi[k] - lo[k] + 1);
    let mut cropped = crate::object_grid::Grid {
        dims,
        cells: vec![false; dims.into_iter().product()],
    };
    for z in 0..dims[2] {
        for y in 0..dims[1] {
            for x in 0..dims[0] {
                let p = [x, y, z];
                let i = cropped.index(p);
                cropped.cells[i] = g.get(std::array::from_fn(|k| (p[k] + lo[k]) as i32));
            }
        }
    }
    let step: V = std::array::from_fn(|k| b.size[k] / g.dims[k] as f32);
    let size: V = std::array::from_fn(|k| step[k] * dims[k] as f32);
    let offset: V = std::array::from_fn(|k| {
        lo[k] as f32 * step[k]
            + if k < 2 {
                size[k] / 2.0 - b.size[k] / 2.0
            } else {
                0.0
            }
    });
    b.at = b.world(offset);
    b.size = size;
    b.grid = Some(cropped);
}
