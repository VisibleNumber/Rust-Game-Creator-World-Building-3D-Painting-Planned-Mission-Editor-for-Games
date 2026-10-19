use super::*;

fn app() -> App {
    App::new(std::env::temp_dir().join(format!("grove-absent-{}.grove", std::process::id())))
}

#[test]
fn editing_history_protects_locked_foundation_and_preserves_group_colors() {
    let mut a = app();
    let initial = a.scene.clone();
    a.selected = vec![0, 6];
    a.action(Action::Duplicate);
    assert_eq!(a.scene.objects.len(), initial.objects.len() + 1);
    let copy = *a.selected.last().unwrap();
    assert_eq!(a.scene.objects[copy].shape, Shape::Steps);
    a.action(Action::Color(4));
    assert_eq!(a.scene.objects[copy].color, COLORS[4]);
    a.action(Action::Undo);
    assert_eq!(a.scene.objects[copy].color, initial.objects[6].color);
    a.action(Action::Undo);
    assert_eq!(a.scene, initial);
    a.action(Action::Redo);
    assert_eq!(a.scene.objects.len(), initial.objects.len() + 1);
    a.selected = vec![0];
    a.action(Action::Delete);
    assert_eq!(a.scene.objects[0], initial.objects[0]);
}

#[test]
fn walk_climbs_stairs_and_ramp_without_falling_through() {
    let mut a = app();
    a.feet = [-13.0, 1.0, 0.0];
    for _ in 0..800 {
        a.walk_step(0.005, [0.0, 3.0, 0.0], false);
    }
    assert!(a.feet[1] > 11.5, "stuck at {:?}", a.feet);
    assert!((a.feet[2] - 2.4).abs() < 0.01);
    a.feet = [-6.0, 1.0, 0.0];
    a.vz = 0.0;
    for _ in 0..540 {
        a.walk_step(0.005, [0.0, 3.0, 0.0], false);
    }
    assert!(a.feet[1] > 8.5, "stuck at {:?}", a.feet);
    assert!(a.feet[2] > 1.7);
}

#[test]
fn walk_stops_at_wall_and_jump_lands_back_on_ground() {
    let mut a = app();
    a.feet = [0.0, 21.0, 0.0];
    for _ in 0..400 {
        a.walk_step(0.005, [0.0, 6.0, 0.0], false);
    }
    assert!(a.feet[1] < 22.8, "walk passed through wall: {:?}", a.feet);
    a.feet = [0.0, -8.0, 0.0];
    a.vz = 0.0;
    a.grounded = true;
    let mut peak = 0.0f32;
    for i in 0..400 {
        a.walk_step(0.005, [0.0; 3], i == 0);
        peak = peak.max(a.feet[2]);
    }
    assert!(peak > 0.8);
    assert!(a.grounded);
    assert!(a.feet[2].abs() < 0.001);
}

#[test]
fn inspector_rejects_invalid_dimensions_without_mutating_scene() {
    let mut a = app();
    a.selected = vec![6];
    a.sync_fields();
    let before = a.scene.clone();
    a.fields[4] = "NaN".into();
    a.apply();
    assert_eq!(a.scene, before);
    a.fields[4] = "6.5".into();
    a.fields[0] = "My stairs".into();
    a.apply();
    assert_eq!(a.scene.objects[6].size[0], 6.5);
    assert_eq!(a.scene.objects[6].name, "My stairs");
    a.action(Action::Undo);
    assert_eq!(a.scene, before);
}

#[test]
fn switching_grab_modes_is_one_undo_and_cancel_restores_everything() {
    let mut a = app();
    a.magnet = false;
    a.selected = vec![6];
    a.tool = Tool::Move;
    a.pointer = [850.0, 500.0];
    let original = a.scene.clone();
    a.begin_grab(true);
    a.drag_to([930.0, 510.0], 1482, 942);
    assert_ne!(a.scene, original);
    a.pointer = [930.0, 510.0];
    a.cycle_mode();
    a.drag_to([1030.0, 510.0], 1482, 942);
    assert!(a.scene.objects[6].yaw.abs() > 1.0);
    a.finish_grab(false);
    assert_eq!(a.undo.len(), 1);
    a.action(Action::Undo);
    assert_eq!(a.scene, original);
    a.selected = vec![6];
    a.begin_grab(true);
    a.drag_to([900.0, 400.0], 1482, 942);
    a.cycle_mode();
    a.finish_grab(true);
    assert_eq!(a.scene, original);
}

#[test]
fn axis_locked_grab_only_changes_that_axis() {
    let mut a = app();
    a.magnet = false;
    a.selected = vec![6];
    a.tool = Tool::Move;
    a.axis = Some(2);
    a.pointer = [850.0, 500.0];
    let before = a.scene.objects[6].at;
    a.begin_grab(true);
    a.drag_to([850.0, 400.0], 1482, 942);
    let after = a.scene.objects[6].at;
    assert_eq!(before[0], after[0]);
    assert_eq!(before[1], after[1]);
    assert!(after[2] > before[2]);
    a.finish_grab(false);
}

#[test]
fn carved_doorway_has_real_geometry_and_walk_collision() {
    let mut a = app();
    a.scene.objects.truncate(1);
    a.scene.objects.push(Object::new(
        "Wall",
        Shape::Box,
        [0.0; 3],
        [4.0, 0.25, 3.0],
        COLORS[1],
    ));
    a.selected = vec![1];
    a.action(Action::Sculpt);
    let start = [-0.6, -0.125, 0.05];
    let end = [0.6, -0.125, 2.1];
    a.begin_cut(unit(sub(start, a.cam.at)));
    assert!(a.carve.is_some());
    let pixel = a.screen(end).unwrap();
    a.update_cut(pixel);
    a.end_cut(false);
    let wall = &a.scene.objects[1];
    assert!(pick(&[wall.clone()], [0.0, -2.0, 1.0], [0.0, 1.0, 0.0]).is_none());
    assert!(pick(&[wall.clone()], [-1.6, -2.0, 1.0], [0.0, 1.0, 0.0]).is_some());
    let cut_grid = wall.grid.clone();
    a.sync_fields();
    a.fields[0] = "Renamed cut wall".into();
    a.apply();
    assert_eq!(a.scene.objects[1].grid, cut_grid);
    a.feet = [0.0, -2.0, 0.0];
    for _ in 0..300 {
        a.walk_step(0.005, [0.0, 3.0, 0.0], false);
    }
    assert!(a.feet[1] > 1.5, "Blocked by carved opening: {:?}", a.feet);
    a.action(Action::Undo);
    a.action(Action::Undo);
    assert!(a.scene.objects[1]
        .grid
        .as_ref()
        .unwrap()
        .cells
        .iter()
        .all(|x| *x));
}

#[test]
fn repeating_wall_parts_touch_and_undo_as_one_operation() {
    let mut a = app();
    a.magnet = false;
    a.action(Action::Kit(0));
    let start = a.scene.objects.len();
    a.line_mode = true;
    a.stamp([0.0, -15.0, 0.0]);
    a.stamp([10.0, -15.0, 0.0]);
    let row = &a.scene.objects[start..];
    assert_eq!(row.len(), 4);
    for pair in row.windows(2) {
        assert!((pair[1].at[0] - pair[0].at[0] - 4.0).abs() < 0.001);
    }
    a.action(Action::Undo);
    assert_eq!(a.scene.objects.len(), start);
}

#[test]
fn world_rotation_preserves_local_geometry_under_multiple_axes() {
    let mut b = Object::new(
        "test",
        Shape::Box,
        [1.0, 2.0, 3.0],
        [2.0, 3.0, 4.0],
        COLORS[0],
    );
    let local = [0.4, -0.7, 1.2];
    let p = b.world(local);
    b.rotate_about([0.0; 3], 0, 37.0);
    b.rotate_about([0.0; 3], 1, 62.0);
    b.rotate_about([0.0; 3], 2, 15.0);
    let expected = rotate(rotate(rotate(p, 0, 37.0), 1, 62.0), 2, 15.0);
    assert!(dot(sub(b.world(local), expected), sub(b.world(local), expected)) < 0.00001);
    let back = b.local(b.world(local));
    assert!(dot(sub(back, local), sub(back, local)) < 0.00001);
}

#[test]
fn joining_and_subtraction_are_single_undo_operations() {
    let mut a = app();
    a.scene.objects.truncate(1);
    a.scene.objects.push(Object::new(
        "body",
        Shape::Box,
        [0.0; 3],
        [4.0; 3],
        COLORS[0],
    ));
    a.scene.objects.push(Object::new(
        "cutter",
        Shape::Cylinder,
        [0.0, 0.0, -1.0],
        [2.0, 2.0, 6.0],
        COLORS[1],
    ));
    let before = a.scene.clone();
    a.selected = vec![2, 1];
    a.action(Action::Subtract);
    assert_eq!(a.scene.objects.len(), 2);
    assert!(!a.scene.objects[1].contains([0.0, 0.0, 2.0]));
    assert!(a.scene.objects[1].contains([1.7, 0.0, 2.0]));
    a.action(Action::Undo);
    assert_eq!(a.scene, before);
    a.selected = vec![1, 2];
    a.action(Action::Combine);
    assert_eq!(a.scene.objects.len(), 2);
    assert!(a.scene.objects[1].contains([0.0, 0.0, 4.5]));
    a.action(Action::Undo);
    assert_eq!(a.scene, before);
}
