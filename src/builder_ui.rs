unsafe fn ui(app: &mut App, font: u32, w: i32, h: i32) {
    render::ui_begin(w, h);
    app.hits.clear();
    render::rect([0.0, 0.0, SIDE as f32, h as f32], [20, 28, 39]);
    render::rect([SIDE as f32, 0.0, w as f32, 53.0], [16, 24, 34]);
    render::text(font, 18.0, 13.0, "PROJECT GROVE", [104, 220, 210]);
    render::text(
        font,
        18.0,
        36.0,
        "BUILD / SHAPE / PLAY   0.3",
        [132, 156, 177],
    );
    let mut x = SIDE as f32 + 12.0;
    for (a, l, on, width) in [
        (Action::Save, "Save", false, 60.0),
        (Action::Load, "Load", false, 60.0),
        (Action::Undo, "Undo", false, 60.0),
        (Action::Redo, "Redo", false, 60.0),
        (Action::Home, "Home", false, 63.0),
        (Action::TopView, "Top", false, 52.0),
        (
            Action::Walk,
            if app.walk {
                "Build [Tab]"
            } else {
                "Walk [Tab]"
            },
            app.walk,
            98.0,
        ),
        (Action::Help, "Help [F1]", app.help, 90.0),
    ] {
        button(app, font, [x, 10.0, x + width, 42.0], l, a, on);
        x += width + 5.0;
    }
    let mut y = 76.0 - app.scroll;
    macro_rules! lab {
        ($s:expr) => {
            if y >= 64.0 && y < h as f32 - 50.0 {
                label(font, y, $s);
            }
            y += 25.0;
        };
    }
    macro_rules! row {
        ($items:expr) => {{
            let items: Vec<(Action, String, bool)> = $items;
            let cw = 288.0 / items.len() as f32;
            for (i, (a, l, on)) in items.into_iter().enumerate() {
                if y >= 64.0 && y + 32.0 < h as f32 - 50.0 {
                    button(
                        app,
                        font,
                        [
                            18.0 + i as f32 * cw,
                            y,
                            12.0 + (i + 1) as f32 * cw,
                            y + 32.0,
                        ],
                        &l,
                        a,
                        on,
                    );
                }
            }
            y += 38.0;
        }};
    }
    lab!("1. SELECT A PIECE, THEN GRAB IT");
    row!(vec![
        (
            Action::Tool(Tool::Select),
            "Select [1]".into(),
            app.tool == Tool::Select
        ),
        (
            Action::Grab,
            if app.drag.is_some() {
                "Place [click]"
            } else {
                "Grab [G]"
            }
            .into(),
            app.drag.is_some()
        )
    ]);
    row!(vec![
        (
            Action::Tool(Tool::Move),
            "Move [3]".into(),
            app.tool == Tool::Move
        ),
        (
            Action::Tool(Tool::Rotate),
            "Turn [R]".into(),
            app.tool == Tool::Rotate
        ),
        (
            Action::Tool(Tool::Scale),
            "Size [5]".into(),
            app.tool == Tool::Scale
        )
    ]);
    row!(vec![
        (Action::FreeAxis, "Free".into(), app.axis.is_none()),
        (Action::Axis(0), "X".into(), app.axis == Some(0)),
        (Action::Axis(1), "Y".into(), app.axis == Some(1)),
        (Action::Axis(2), "Z".into(), app.axis == Some(2))
    ]);
    row!(vec![
        (
            Action::Grid,
            format!("Snap {}m [ ]", GRIDS[app.grid]),
            false
        ),
        (
            Action::ShowGrid,
            format!("Grid {} [B]", if app.grid_visible { "ON" } else { "OFF" }),
            app.grid_visible
        )
    ]);
    row!(vec![
        (
            Action::Magnet,
            format!("Join {} [N]", if app.magnet { "ON" } else { "OFF" }),
            app.magnet
        ),
        (
            Action::RotateSnap,
            if app.angle {
                "Turn snap 15"
            } else {
                "Turn free"
            }
            .into(),
            app.angle
        ),
        (
            Action::GridOrigin,
            "Origin".into(),
            app.grid_origin != [0.0; 3]
        )
    ]);
    lab!("2. SHAPE YOUR OWN OBJECT");
    row!(vec![
        (Action::Combine, "Join pieces".into(), false),
        (Action::Subtract, "Use as cutters".into(), false)
    ]);
    row!(vec![(
        Action::Sculpt,
        if app.sculpting {
            "Finish object [E]"
        } else {
            "Edit object [E]"
        }
        .into(),
        app.sculpting
    )]);
    if app.sculpting {
        row!(vec![
            (
                Action::ObjectMode(0),
                "Cut [C]".into(),
                !app.fill && !app.extrude
            ),
            (
                Action::ObjectMode(1),
                "Add [C]".into(),
                app.fill && !app.extrude
            ),
            (Action::ObjectMode(2), "Pull out [P]".into(), app.extrude)
        ]);
        lab!("CHOOSE YOUR BRUSH / V CYCLES");
        row!(vec![
            (
                Action::Brush(Brush::Rectangle),
                "Box".into(),
                app.brush == Brush::Rectangle
            ),
            (
                Action::Brush(Brush::Ellipse),
                "Circle".into(),
                app.brush == Brush::Ellipse
            ),
            (
                Action::Brush(Brush::Arch),
                "Arch".into(),
                app.brush == Brush::Arch
            )
        ]);
        row!(vec![
            (
                Action::Brush(Brush::Triangle),
                "Triangle".into(),
                app.brush == Brush::Triangle
            ),
            (
                Action::Brush(Brush::Diamond),
                "Diamond".into(),
                app.brush == Brush::Diamond
            ),
            (
                Action::Brush(Brush::Ball),
                "Ball".into(),
                app.brush == Brush::Ball
            )
        ]);
        row!(vec![(
            Action::Brush(Brush::Freehand),
            "Draw freehand".into(),
            app.brush == Brush::Freehand
        )]);
        lab!(&format!("DEPTH: {} CELLS / [ ] OR WHEEL", app.depth));
        row!(vec![
            (Action::Depth(-1), "Less".into(), false),
            (Action::Depth(1), "More".into(), false),
            (
                Action::Through,
                if app.through {
                    "Through ON"
                } else {
                    "Depth only"
                }
                .into(),
                app.through && !app.extrude
            )
        ]);
        if app.brush == Brush::Freehand {
            lab!(&format!("DRAW WIDTH: {} CELLS", app.brush_radius * 2 + 1));
            row!(vec![
                (Action::Radius(-1), "Thinner".into(), false),
                (Action::Radius(1), "Thicker".into(), false)
            ]);
        }
        lab!("MIRROR / X Y Z TOGGLES");
        row!(vec![
            (Action::Mirror(0), "Mirror X".into(), app.mirror & 1 != 0),
            (Action::Mirror(1), "Mirror Y".into(), app.mirror & 2 != 0),
            (Action::Mirror(2), "Mirror Z".into(), app.mirror & 4 != 0)
        ]);
        row!(vec![
            (
                Action::SculptGrid,
                format!("Grid {}", app.sculpt_resolution),
                false
            ),
            (Action::Pad, "Expand space".into(), false)
        ]);
        row!(vec![
            (Action::Hollow, "Hollow object".into(), false),
            (Action::RoundEdges, "Round edges".into(), false)
        ]);
        if let Some(b) = app.primary() {
            if let Some(g) = &b.grid {
                let dims = g.dims;
                let step = b.size[0] / dims[0] as f32;
                lab!(&format!(
                    "CANVAS {} x {} x {} / {:.3}m",
                    dims[0], dims[1], dims[2], step
                ));
            }
        }
    }
    if !app.sculpting {
        lab!("3. READY-TO-BUILD PARTS");
        for i in (0..16).step_by(2) {
            row!(vec![
                (Action::Kit(i), crate::building_kit::NAMES[i].into(), false),
                (
                    Action::Kit(i + 1),
                    crate::building_kit::NAMES[i + 1].into(),
                    false
                )
            ]);
        }
        row!(vec![
            (
                Action::Repeat,
                format!("Repeat line {}", if app.line_mode { "ON" } else { "OFF" }),
                app.line_mode
            ),
            (
                Action::Tool(Tool::Place),
                "Stamp [2]".into(),
                app.tool == Tool::Place
            )
        ]);
        lab!("BASIC SHAPES");
        row!(vec![
            (Action::Shape(Shape::Box), "Block".into(), false),
            (Action::Shape(Shape::Ramp), "Ramp".into(), false),
            (Action::Shape(Shape::Roof), "Roof".into(), false)
        ]);
        row!(vec![
            (Action::Shape(Shape::Steps), "Stairs".into(), false),
            (Action::Shape(Shape::Cylinder), "Cylinder".into(), false),
            (Action::Shape(Shape::Sphere), "Sphere".into(), false)
        ]);
    }
    lab!("COLOR");
    for i in 0..10 {
        if y >= 64.0 && y + 25.0 < h as f32 - 50.0 {
            let x = 18.0 + i as f32 * 28.5;
            let r = [x, y, x + 24.0, y + 24.0];
            render::rect(r, COLORS[i]);
            app.hits.push(Hit {
                rect: r,
                action: Action::Color(i),
            });
        }
    }
    y += 36.0;
    row!(vec![
        (Action::Duplicate, "Copy [Ctrl+D]".into(), false),
        (Action::Delete, "Delete".into(), false),
        (Action::Focus, "Focus [F]".into(), false)
    ]);
    row!(vec![
        (Action::SavePrefab, "Save build".into(), false),
        (Action::LoadPrefab, "Load build".into(), false)
    ]);
    row!(vec![(
        Action::Properties,
        if app.inspector {
            "Hide properties"
        } else {
            "Exact properties"
        }
        .into(),
        app.inspector
    )]);
    if app.inspector {
        for (i, title) in [
            (0, "NAME"),
            (1, "POSITION X"),
            (2, "POSITION Y"),
            (3, "POSITION Z"),
            (4, "WIDTH X"),
            (5, "DEPTH Y"),
            (6, "HEIGHT Z"),
            (7, "TURN Z"),
            (11, "TILT X"),
            (12, "TILT Y"),
            (8, "RED 0-255"),
            (9, "GREEN 0-255"),
            (10, "BLUE 0-255"),
        ] {
            if y >= 64.0 && y + 27.0 < h as f32 - 50.0 {
                render::text(font, 18.0, y + 4.0, title, [144, 164, 183]);
                let r = [133.0, y, 300.0, y + 27.0];
                render::rect(
                    r,
                    if app.active == Some(i) {
                        [45, 90, 104]
                    } else {
                        [33, 45, 59]
                    },
                );
                render::text(
                    font,
                    140.0,
                    y + 4.0,
                    &app.fields[i].chars().take(22).collect::<String>(),
                    [230, 236, 242],
                );
                app.hits.push(Hit {
                    rect: r,
                    action: Action::Field(i),
                });
            }
            y += 31.0;
        }
        row!(vec![(Action::Apply, "Apply values [Enter]".into(), false)]);
    }
    let solid = app.primary().is_none_or(|b| b.solid);
    let locked = app.primary().is_some_and(|b| b.locked);
    row!(vec![
        (
            Action::Collision,
            format!("Solid {}", if solid { "ON" } else { "OFF" }),
            solid
        ),
        (
            Action::Lock,
            if locked { "Unlock" } else { "Lock" }.into(),
            locked
        )
    ]);
    row!(vec![
        (Action::Spawn, "Set player spawn".into(), false),
        (Action::Animate, "Animate dummy".into(), app.animate)
    ]);
    row!(vec![
        (Action::Blank, "Empty yard".into(), false),
        (Action::Starter, "Starter yard".into(), false)
    ]);
    app.scroll = app
        .scroll
        .clamp(0.0, (y + app.scroll - h as f32 + 65.0).max(0.0));
    // The active instructions stay visible even while the toolbox is scrolled.
    if !app.walk {
        let mode = if app.sculpting {
            if app.fill {
                "FILL SHAPE"
            } else {
                "CUT SHAPE"
            }
        } else if app.drag.is_some() {
            match app.tool {
                Tool::Rotate => "HOLDING / TURN",
                Tool::Scale => "HOLDING / SIZE",
                _ => "HOLDING / MOVE",
            }
        } else if app.tool == Tool::Place {
            "STAMP A PIECE"
        } else {
            "SELECT / BUILD"
        };
        render::rect(
            [
                SIDE as f32 + 12.0,
                65.0,
                (SIDE + 535).min(w - 8) as f32,
                133.0,
            ],
            [19, 34, 46],
        );
        render::text(
            font,
            SIDE as f32 + 24.0,
            73.0,
            &format!(
                "{}   |   AXIS {}   |   SNAP {}m",
                mode,
                app.axis_label(),
                GRIDS[app.grid]
            ),
            [110, 230, 206],
        );
        render::text(
            font,
            SIDE as f32 + 24.0,
            97.0,
            if app.sculpting {
                "Drag to shape | C: cut/add | P: pull | V: brush | X/Y/Z: mirror"
            } else if app.drag.is_some() {
                "R: change mode | X/Y/Z: axis | Click: place | Esc: cancel"
            } else if app.tool == Tool::Place {
                "Click: stamp | R: turn 90 | Wheel: fine turn | Esc: select"
            } else {
                "Click a piece -> G to grab | R changes mode | F1: help"
            },
            [198, 215, 228],
        );
    }
    // Axis labels are clickable handles and remain visible over the selection.
    if !app.sculpting {
        if let Some(b) = app.primary() {
            let pivot = b.at;
            let length =
                (dot(sub(pivot, app.cam.at), sub(pivot, app.cam.at)).sqrt() * 0.13).clamp(1.0, 7.0);
            for k in 0..3 {
                let mut p = pivot;
                p[k] += length;
                if let Some(q) = app.screen(p) {
                    if q[0] > SIDE as f32 + 5.0 && q[1] > 140.0 && q[1] < h as f32 - 70.0 {
                        button(
                            app,
                            font,
                            [q[0] - 13.0, q[1] - 13.0, q[0] + 17.0, q[1] + 17.0],
                            ["X", "Y", "Z"][k],
                            Action::Axis(k),
                            app.axis == Some(k),
                        );
                    }
                }
            }
        }
    }
    render::rect([0.0, h as f32 - 46.0, w as f32, h as f32], [13, 20, 29]);
    render::text(
        font,
        18.0,
        h as f32 - 39.0,
        &format!(
            "{} pieces | {} | {}",
            app.scene.objects.len(),
            if app.dirty { "UNSAVED" } else { "SAVED" },
            app.status
                .chars()
                .take(((w - 300).max(180) / 7) as usize)
                .collect::<String>()
        ),
        [188, 215, 223],
    );
    render::text(font,18.0,h as f32-19.0,"Right mouse + WASD: fly | G: grab | R: mode | X/Y/Z: axis | E: edit shape | [ ]: snap size | F1: help | Scroll toolbox for more",[123,145,167]);
    if app.help {
        let left = SIDE as f32 + 35.0;
        let top = 155.0;
        render::rect(
            [left, top, (left + 630.0).min(w as f32 - 15.0), top + 480.0],
            [16, 28, 39],
        );
        for (i, s) in [
            "BUILDING QUICK GUIDE",
            "Click a piece, press G, then move the mouse.",
            "R cycles MOVE -> TURN -> SIZE while holding it.",
            "X / Y / Z locks an axis; press it again for free movement.",
            "Click or Enter places; Escape restores the original piece.",
            "Hold Shift for fine adjustments. Right mouse + WASD flies.",
            "[ / ] changes world snapping; B shows grid; N joins edges.",
            "Choose a ready part, then click to stamp. R turns it 90 deg.",
            "Repeat line: click START then END to stamp a row.",
            "E opens the object editor. V cycles brushes.",
            "C: cut/add. P: pull out. X/Y/Z: mirror while editing.",
            "[ / ]: depth. Circle, arch, triangle, ball and freehand brushes.",
            "Expand space adds empty canvas. Hollow / Round refine it.",
            "Ctrl+click adds pieces. Ctrl+D copies. Ctrl+Z / Y undo/redo.",
            "Save build stores your selection; Load build stamps it.",
            "Tab: walk the world. Space: jump. F: focus. F12: photo.",
            "Scroll the toolbox for colors, properties, and saved builds.",
            "F1 or click the world closes this guide.",
        ]
        .iter()
        .enumerate()
        {
            render::text(
                font,
                left + 18.0,
                top + 16.0 + i as f32 * 24.0,
                s,
                if i == 0 {
                    [110, 230, 206]
                } else {
                    [209, 224, 233]
                },
            );
        }
    }
}
