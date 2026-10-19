impl App {
    fn combine_objects(&mut self) {
        let ids: Vec<_> = self
            .selected
            .iter()
            .copied()
            .filter(|&i| !self.scene.objects[i].locked)
            .collect();
        if ids.len() < 2 {
            self.status = "Ctrl+click at least two unlocked pieces, then Join pieces".into();
            return;
        }
        let pieces: Vec<_> = ids.iter().map(|&i| self.scene.objects[i].clone()).collect();
        let (lo, hi) = crate::building_kit::bounds(&pieces);
        let size = sub(hi, lo);
        let at = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, lo[2]];
        let mut joined = Object::new(
            "Joined custom object",
            Shape::Box,
            at,
            size,
            pieces.last().unwrap().color,
        );
        if !joined.valid() {
            self.status = "Selection is too large to join as one object".into();
            return;
        }
        let mut grid =
            crate::object_grid::Grid::from_object(&joined, self.sculpt_resolution.min(32));
        for z in 0..grid.dims[2] {
            for y in 0..grid.dims[1] {
                for x in 0..grid.dims[0] {
                    let p = [x, y, z];
                    let q: V = std::array::from_fn(|k| {
                        (p[k] as f32 + 0.5) / grid.dims[k] as f32 * size[k]
                            - if k < 2 { size[k] / 2.0 } else { 0.0 }
                    });
                    let i = grid.index(p);
                    let world = joined.world(q);
                    grid.cells[i] = pieces.iter().any(|b| b.contains(world));
                }
            }
        }
        if !grid.cells.iter().any(|v| *v) {
            self.status = "No material at this grid resolution. Try a finer object grid.".into();
            return;
        }
        joined.grid = Some(grid);
        let before = self.scene.clone();
        self.scene.objects = self
            .scene
            .objects
            .iter()
            .enumerate()
            .filter(|(i, _)| !ids.contains(i))
            .map(|(_, b)| b.clone())
            .collect();
        let index = self.scene.objects.len();
        self.scene.objects.push(joined);
        self.selected = vec![index];
        self.sculpting = true;
        self.tool = Tool::Select;
        self.checkpoint(before);
        self.status="Joined as one editable object. Fine details use the selected grid; Ctrl+Z restores the pieces.".into();
    }
    fn subtract_objects(&mut self) {
        let Some(&target) = self.selected.last() else {
            return;
        };
        if self.scene.objects[target].locked {
            self.status = "The last selected object must be unlocked".into();
            return;
        }
        let cutters: Vec<_> = self
            .selected
            .iter()
            .copied()
            .filter(|&i| i != target && !self.scene.objects[i].locked)
            .collect();
        if cutters.is_empty() {
            self.status =
                "Select cutter shapes first, then Ctrl+click the object to cut last".into();
            return;
        }
        let mut b = self.scene.objects[target].clone();
        let mut grid = b.grid.clone().unwrap_or_else(|| {
            crate::object_grid::Grid::from_object(&b, self.sculpt_resolution.min(32))
        });
        let mut count = 0;
        for z in 0..grid.dims[2] {
            for y in 0..grid.dims[1] {
                for x in 0..grid.dims[0] {
                    let p = [x, y, z];
                    let i = grid.index(p);
                    if !grid.cells[i] {
                        continue;
                    }
                    let q: V = std::array::from_fn(|k| {
                        (p[k] as f32 + 0.5) / grid.dims[k] as f32 * b.size[k]
                            - if k < 2 { b.size[k] / 2.0 } else { 0.0 }
                    });
                    let world = b.world(q);
                    if cutters
                        .iter()
                        .any(|&i| self.scene.objects[i].contains(world))
                    {
                        grid.cells[i] = false;
                        count += 1;
                    }
                }
            }
        }
        if count == 0 {
            self.status =
                "The cutter shapes do not overlap filled cells in the last selected object".into();
            return;
        }
        let before = self.scene.clone();
        b.grid = Some(grid);
        self.scene.objects[target] = b;
        let new_index = target - cutters.iter().filter(|&&i| i < target).count();
        self.scene.objects = self
            .scene
            .objects
            .iter()
            .enumerate()
            .filter(|(i, _)| !cutters.contains(i))
            .map(|(_, b)| b.clone())
            .collect();
        self.selected = vec![new_index];
        self.sculpting = true;
        self.tool = Tool::Select;
        self.checkpoint(before);
        self.status=format!("Subtracted {count} cells; cutter pieces consumed. Ctrl+Z restores the whole operation.");
    }
}
