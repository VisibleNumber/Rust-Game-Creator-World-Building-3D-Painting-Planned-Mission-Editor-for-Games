use crate::scene::*;
pub const NAMES: [&str; 16] = [
    "Wall 4m",
    "Floor 4x4",
    "Door wall",
    "Window wall",
    "Pitched roof",
    "Room 8x8",
    "House shell",
    "Garage shell",
    "Road 8x12",
    "Pavement",
    "Curb",
    "Crossroad",
    "T junction",
    "Fence 4m",
    "Stairs",
    "Ramp",
];
pub fn pieces(which: usize) -> Vec<Object> {
    let mut v = Vec::new();
    let mut put =
        |name: &str, shape, at, size, color| v.push(Object::new(name, shape, at, size, color));
    let wall = COLORS[8];
    let roof = COLORS[4];
    let road = [61, 68, 78];
    let pavement = COLORS[0];
    match which {
        0 => put("Wall", Shape::Box, [0.0; 3], [4.0, 0.2, 3.0], wall),
        1 => put("Floor", Shape::Box, [0.0; 3], [4.0, 4.0, 0.2], pavement),
        2 => {
            put(
                "Door left",
                Shape::Box,
                [-1.25, 0.0, 0.0],
                [1.5, 0.2, 3.0],
                wall,
            );
            put(
                "Door right",
                Shape::Box,
                [1.25, 0.0, 0.0],
                [1.5, 0.2, 3.0],
                wall,
            );
            put(
                "Door lintel",
                Shape::Box,
                [0.0, 0.0, 2.2],
                [1.0, 0.2, 0.8],
                wall,
            );
        }
        3 => {
            for x in [-1.5, 1.5] {
                put(
                    "Window side",
                    Shape::Box,
                    [x, 0.0, 0.0],
                    [1.0, 0.2, 3.0],
                    wall,
                );
            }
            put(
                "Window sill",
                Shape::Box,
                [0.0, 0.0, 0.0],
                [2.0, 0.2, 1.0],
                wall,
            );
            put(
                "Window lintel",
                Shape::Box,
                [0.0, 0.0, 2.2],
                [2.0, 0.2, 0.8],
                wall,
            );
        }
        4 => put("Pitched roof", Shape::Roof, [0.0; 3], [8.4, 8.4, 2.0], roof),
        5 | 6 | 7 => {
            put("Floor", Shape::Box, [0.0; 3], [8.0, 8.0, 0.2], pavement);
            put(
                "Back wall",
                Shape::Box,
                [0.0, 3.9, 0.2],
                [8.0, 0.2, 3.0],
                wall,
            );
            for x in [-3.9, 3.9] {
                put(
                    "Side wall",
                    Shape::Box,
                    [x, 0.0, 0.2],
                    [0.2, 7.6, 3.0],
                    wall,
                );
            }
            let gap = if which == 7 { 3.2 } else { 1.2 };
            let side = (8.0 - gap) / 2.0;
            for x in [-1.0, 1.0] {
                put(
                    "Front wall",
                    Shape::Box,
                    [x * (gap / 2.0 + side / 2.0), -3.9, 0.2],
                    [side, 0.2, 3.0],
                    wall,
                );
            }
            put(
                "Entry lintel",
                Shape::Box,
                [0.0, -3.9, 2.5],
                [gap, 0.2, 0.7],
                wall,
            );
            if which == 6 {
                put(
                    "House roof",
                    Shape::Roof,
                    [0.0, 0.0, 3.2],
                    [8.6, 8.6, 2.0],
                    roof,
                );
            }
            if which == 7 {
                put(
                    "Garage roof",
                    Shape::Box,
                    [0.0, 0.0, 3.2],
                    [8.4, 8.4, 0.2],
                    pavement,
                );
            }
        }
        8 => {
            put("Road", Shape::Box, [0.0; 3], [8.0, 12.0, 0.16], road);
            for y in [-4.5, -1.5, 1.5, 4.5] {
                put(
                    "Center dash",
                    Shape::Box,
                    [0.0, y, 0.165],
                    [0.12, 1.4, 0.01],
                    [223, 203, 120],
                );
            }
        }
        9 => put("Pavement", Shape::Box, [0.0; 3], [2.0, 12.0, 0.3], pavement),
        10 => put(
            "Curb",
            Shape::Box,
            [0.0; 3],
            [0.2, 12.0, 0.3],
            [210, 210, 202],
        ),
        11 | 12 => {
            put(
                "Junction center",
                Shape::Box,
                [0.0; 3],
                [8.0, 8.0, 0.16],
                road,
            );
            for x in [-6.0, 6.0] {
                put(
                    "Cross street",
                    Shape::Box,
                    [x, 0.0, 0.0],
                    [4.0, 8.0, 0.16],
                    road,
                );
            }
            put(
                "South street",
                Shape::Box,
                [0.0, -6.0, 0.0],
                [8.0, 4.0, 0.16],
                road,
            );
            if which == 11 {
                put(
                    "North street",
                    Shape::Box,
                    [0.0, 6.0, 0.0],
                    [8.0, 4.0, 0.16],
                    road,
                );
            }
        }
        13 => {
            for x in [-1.9, 1.9] {
                put(
                    "Fence post",
                    Shape::Box,
                    [x, 0.0, 0.0],
                    [0.2, 0.2, 1.4],
                    wall,
                );
            }
            for z in [0.45, 1.05] {
                put(
                    "Fence rail",
                    Shape::Box,
                    [0.0, 0.0, z],
                    [4.0, 0.12, 0.15],
                    wall,
                );
            }
        }
        14 => put("Stairs", Shape::Steps, [0.0; 3], [1.5, 4.0, 2.0], COLORS[1]),
        _ => put("Ramp", Shape::Ramp, [0.0; 3], [3.0, 6.0, 2.0], COLORS[2]),
    }
    normalize(&mut v);
    v
}
pub fn bounds(objects: &[Object]) -> (V, V) {
    let mut lo = [f32::INFINITY; 3];
    let mut hi = [f32::NEG_INFINITY; 3];
    for b in objects {
        let (a, z) = b.bounds();
        for k in 0..3 {
            lo[k] = lo[k].min(a[k]);
            hi[k] = hi[k].max(z[k]);
        }
    }
    (lo, hi)
}
pub fn normalize(objects: &mut [Object]) -> V {
    let (lo, hi) = bounds(objects);
    if objects.is_empty() {
        return [1.0; 3];
    }
    let center = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, lo[2]];
    for b in objects {
        b.at = sub(b.at, center);
        b.locked = false;
    }
    sub(hi, lo)
}
