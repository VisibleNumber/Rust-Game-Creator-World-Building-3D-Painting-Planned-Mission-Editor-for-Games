impl App {
    fn scroll_world(&mut self, n: f32) {
        if self.sculpting {
            if self.brush == Brush::Freehand {
                self.brush_radius = (self.brush_radius as i32 + n as i32).clamp(0, 8) as usize;
            } else {
                self.depth = (self.depth as i32 + n as i32).clamp(1, 16) as usize;
            }
            return;
        }
        if self.drag.is_some() {
            let pivot = self.primary().map(|b| b.at).unwrap_or([0.0; 3]);
            let before = self.scene.clone();
            for &i in &self.selected {
                if !before.objects[i].locked {
                    self.scene.objects[i].rotate_about(
                        pivot,
                        self.axis.unwrap_or(2),
                        n * if self.fine { 1.0 } else { 15.0 },
                    );
                }
            }
            self.switch_grab(self.tool);
            self.sync_fields();
        } else if self.tool == Tool::Place {
            if self.fine {
                self.placement_height += n * GRIDS[self.grid].max(0.1);
            } else {
                self.placement_yaw = (self.placement_yaw + n * 15.0).rem_euclid(360.0);
            }
        } else if !self.sculpting {
            let (f, _, _) = self.cam.basis();
            self.cam.at = add(self.cam.at, mul(f, n * 2.0));
        }
    }
    fn finish_grab(&mut self, cancel: bool) {
        if let Some(d) = self.drag.take() {
            if cancel {
                self.scene = d.original;
                self.status = "Grab cancelled; everything restored".into();
            } else {
                self.checkpoint(d.original);
                self.status = "Placed. G grabs again; R cycles move / turn / resize.".into();
            }
            self.sync_fields();
            unsafe {
                ReleaseCapture();
            }
        }
    }
    fn begin_grab(&mut self, latched: bool) {
        if self.walk {
            return;
        }
        if self.selected.is_empty() {
            let d = self.ray(self.pointer, self.viewport[0], self.viewport[1]);
            if let Some((i, _)) = pick(&self.scene.objects, self.cam.at, d) {
                self.selected = vec![i];
            }
        }
        if self.selected.iter().all(|&i| self.scene.objects[i].locked) {
            self.status = "Select an unlocked piece first".into();
            return;
        }
        let Some(b) = self.primary() else {
            self.status = "Click a piece, then press G to grab it".into();
            return;
        };
        let pivot = b.at;
        let d = self.ray(self.pointer, self.viewport[0], self.viewport[1]);
        let point = if d[2].abs() > 0.001 {
            add(self.cam.at, mul(d, (pivot[2] - self.cam.at[2]) / d[2]))
        } else {
            pivot
        };
        self.drag = Some(Drag {
            original: self.scene.clone(),
            before: self.scene.clone(),
            start: self.pointer,
            point,
            pivot,
            latched,
        });
        self.sync_fields();
        self.status =
            "HOLDING: mouse adjusts | R changes mode | X/Y/Z axis | click places | Esc cancels"
                .into();
    }
    fn switch_grab(&mut self, tool: Tool) {
        if let Some(mut d) = self.drag.take() {
            d.before = self.scene.clone();
            d.start = self.pointer;
            d.pivot = self.primary().map_or(d.pivot, |b| b.at);
            let ray = self.ray(self.pointer, self.viewport[0], self.viewport[1]);
            d.point = if ray[2].abs() > 0.001 {
                add(
                    self.cam.at,
                    mul(ray, (d.pivot[2] - self.cam.at[2]) / ray[2]),
                )
            } else {
                d.pivot
            };
            self.drag = Some(d);
        }
        self.tool = tool;
    }
    fn cycle_mode(&mut self) {
        if self.tool == Tool::Place {
            self.placement_yaw = (self.placement_yaw + 90.0).rem_euclid(360.0);
            self.status = "Placement rotated 90 degrees. Wheel turns in smaller steps.".into();
            return;
        }
        let next = match self.tool {
            Tool::Move => Tool::Rotate,
            Tool::Rotate => Tool::Scale,
            _ => Tool::Move,
        };
        self.switch_grab(next);
        if self.drag.is_none() {
            self.begin_grab(true);
        }
    }
    fn screen(&self, p: V) -> Option<[f32; 2]> {
        self.cam
            .project(p, [(self.viewport[0] - SIDE).max(1), self.viewport[1]])
            .map(|s| [s[0] + SIDE as f32, s[1]])
    }
    fn axis_label(&self) -> &'static str {
        match self.axis {
            Some(0) => "X",
            Some(1) => "Y",
            Some(2) => "Z",
            _ => "FREE",
        }
    }
    fn build_action(&mut self, a: Action) {
        match a {
            Action::Combine=>self.combine_objects(),Action::Subtract=>self.subtract_objects(),
            Action::Properties => {
                self.inspector = !self.inspector;
            }
            Action::Sculpt | Action::Fill | Action::SculptGrid | Action::Through => {
                self.sculpt_action(a)
            }
            Action::ObjectMode(_)
            | Action::Brush(_)
            | Action::Depth(_)
            | Action::Radius(_)
            | Action::Mirror(_)
            | Action::Hollow
            | Action::RoundEdges
            | Action::Pad => self.sculpt_action(a),
            Action::Grab => {
                if self.drag.is_some() {
                    self.finish_grab(false);
                } else {
                    self.tool = Tool::Move;
                    self.begin_grab(true);
                }
            }
            Action::Axis(k) => {
                self.axis = if self.axis == Some(k) { None } else { Some(k) };
                self.switch_grab(self.tool);
            }
            Action::FreeAxis => {
                self.axis = None;
                self.switch_grab(self.tool);
            }
            Action::ShowGrid => self.grid_visible = !self.grid_visible,
            Action::GridOrigin => {
                self.grid_origin = if self.grid_origin == [0.0; 3] {
                    self.primary().map_or([0.0; 3], |b| b.at)
                } else {
                    [0.0; 3]
                };
                self.status =
                    "Grid origin aligned. Select a piece and use Origin to align to its anchor."
                        .into();
            }
            Action::Magnet => self.magnet = !self.magnet,
            Action::Kit(i) => {
                self.template = crate::building_kit::pieces(i);
                self.template_size = crate::building_kit::normalize(&mut self.template);
                self.size = self.template_size;
                self.placement_yaw = 0.0;
                self.placement_height = 0.0;
                self.tool = Tool::Place;
                self.selected.clear();
                self.line_start = None;
                self.scroll = 0.0;
                self.sync_fields();
                self.status = format!(
                    "{} ready. Click places; R turns 90 degrees; wheel fine-turns.",
                    crate::building_kit::NAMES[i]
                );
            }
            Action::Repeat => {
                self.line_mode = !self.line_mode;
                self.line_start = None;
                self.tool = Tool::Place;
                self.status =
                    "Repeat: click START, then END. Pieces fill the line; Esc cancels.".into();
            }
            Action::SavePrefab => {
                let mut objects: Vec<_> = self
                    .selected
                    .iter()
                    .map(|&i| self.scene.objects[i].clone())
                    .collect();
                if objects.is_empty() {
                    self.status = "Select pieces first (Ctrl + click adds pieces)".into();
                    return;
                }
                crate::building_kit::normalize(&mut objects);
                let s = Scene {
                    objects,
                    spawn: [0.0; 3],
                };
                let dir = self
                    .path
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("../prefabs");
                if let Err(e) = std::fs::create_dir_all(&dir) {
                    self.status = format!("Cannot create building library: {e}");
                    return;
                }
                let Some(file) = choose_build(true, &dir) else {
                    return;
                };
                self.status = match s.save(&file) {
                    Ok(()) => format!(
                        "Building saved: {}. Load build to stamp it.",
                        file.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    Err(e) => format!("Prefab save failed: {e}"),
                };
            }
            Action::LoadPrefab => {
                let dir = self
                    .path
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("../prefabs");
                if let Err(e) = std::fs::create_dir_all(&dir) {
                    self.status = format!("Cannot open building library: {e}");
                    return;
                }
                let Some(file) = choose_build(false, &dir) else {
                    return;
                };
                match Scene::load(&file) {
                    Ok(s) if !s.objects.is_empty() => {
                        self.template = s.objects;
                        self.template_size = crate::building_kit::normalize(&mut self.template);
                        self.size = self.template_size;
                        self.selected.clear();
                        self.placement_yaw = 0.0;
                        self.placement_height = 0.0;
                        self.tool = Tool::Place;
                        self.sync_fields();
                        self.status = "Your saved building is ready to stamp".into();
                    }
                    Ok(_) => self.status = "Saved building is empty".into(),
                    Err(e) => self.status = format!("No saved building yet: {e}"),
                }
            }
            Action::Help => self.help = !self.help,
            Action::TopView => {
                let p =
                    self.primary()
                        .map(|b| b.at)
                        .unwrap_or([self.cam.at[0], self.cam.at[1], 0.0]);
                self.cam.at = add(p, [0.0, -0.1, 35.0]);
                self.cam.pitch = -1.56;
                self.cam.yaw = std::f32::consts::FRAC_PI_2;
                self.walk = false;
            }
            _ => {}
        }
    }
    fn placement(&self, point: V) -> Vec<Object> {
        let at = [
            self.grid_origin[0] + self.snap(point[0] - self.grid_origin[0]),
            self.grid_origin[1] + self.snap(point[1] - self.grid_origin[1]),
            ((point[2] + self.placement_height) * 1000.0).round() / 1000.0,
        ];
        let mut v = if self.template.is_empty() {
            vec![Object::new(
                self.shape.name(),
                self.shape,
                [0.0; 3],
                self.size,
                self.color,
            )]
        } else {
            self.template.clone()
        };
        if !self.template.is_empty() {
            let scale: V = std::array::from_fn(|k| self.size[k] / self.template_size[k].max(0.05));
            for b in &mut v {
                b.at = std::array::from_fn(|k| b.at[k] * scale[k]);
                b.size = std::array::from_fn(|k| (b.size[k] * scale[k]).clamp(0.05, 1000.0));
            }
        }
        for b in &mut v {
            b.rotate_about([0.0; 3], 2, self.placement_yaw);
            b.at = add(b.at, at);
        }
        if self.magnet && !self.fine {
            let d = edge_delta(
                &self.scene.objects,
                &v,
                &[],
                None,
                GRIDS[self.grid].max(0.25) * 0.42,
            );
            for b in &mut v {
                b.at = add(b.at, d);
            }
        }
        v
    }
    fn stamp(&mut self, at: V) {
        let mut pieces = Vec::new();
        if self.line_mode {
            if let Some(start) = self.line_start {
                let delta = sub(at, start);
                let distance = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
                if distance < 0.1 {
                    self.status = "Choose a different end point".into();
                    return;
                }
                let dir = mul([delta[0], delta[1], 0.0], 1.0 / distance);
                let preview = self.placement(start);
                let (lo, hi) = crate::building_kit::bounds(&preview);
                let size = sub(hi, lo);
                let step = (dir[0].abs() * size[0] + dir[1].abs() * size[1]).max(0.1);
                let count = (distance / step).round().max(1.0) as usize + 1;
                if count > 256 {
                    self.status = "Line too long: at most 256 stamps per line".into();
                    return;
                }
                for i in 0..count {
                    pieces.extend(self.placement(add(start, mul(dir, i as f32 * step))));
                }
                self.line_start = None;
            } else {
                self.line_start = Some(at);
                self.status = "Line START set. Now click END; R still turns the pieces.".into();
                return;
            }
        } else {
            pieces = self.placement(at);
        }
        if pieces.iter().any(|b| !b.valid()) || self.scene.objects.len() + pieces.len() > 50000 {
            self.status = "Placement exceeds valid size or scene limits".into();
            return;
        }
        let before = self.scene.clone();
        self.selected =
            (self.scene.objects.len()..self.scene.objects.len() + pieces.len()).collect();
        for mut b in pieces {
            b.name = format!(
                "{} {}",
                b.name.chars().take(65).collect::<String>(),
                self.scene.objects.len() + 1
            );
            self.scene.objects.push(b);
        }
        self.checkpoint(before);
        self.status =
            "Placed! Keep clicking to stamp, or press 1 to select. Ctrl+Z undoes the whole stamp."
                .into();
    }
}
fn edge_delta(
    world: &[Object],
    moving: &[Object],
    exclude: &[usize],
    axis: Option<usize>,
    tolerance: f32,
) -> V {
    if moving.is_empty() {
        return [0.0; 3];
    }
    let (lo, hi) = crate::building_kit::bounds(moving);
    let mut delta = [0.0; 3];
    let mut best = [tolerance; 3];
    for (i, b) in world.iter().enumerate() {
        if exclude.contains(&i) {
            continue;
        }
        let (a, z) = b.bounds();
        if hi[2] < a[2] - 0.1 || lo[2] > z[2] + 0.1 {
            continue;
        }
        for k in 0..2 {
            if axis.is_some_and(|v| v != k) {
                continue;
            }
            let j = 1 - k;
            if hi[j] < a[j] - tolerance || lo[j] > z[j] + tolerance {
                continue;
            }
            for p in [lo[k], hi[k]] {
                for q in [a[k], z[k]] {
                    let d = q - p;
                    if d.abs() < best[k] {
                        best[k] = d.abs();
                        delta[k] = d;
                    }
                }
            }
        }
    }
    delta
}
