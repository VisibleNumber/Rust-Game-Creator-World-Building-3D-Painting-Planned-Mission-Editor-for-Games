use crate::scene::*;
use std::ffi::c_void;
#[link(name = "opengl32")]
extern "system" {
    fn glClearColor(r: f32, g: f32, b: f32, a: f32);
    fn glClear(mask: u32);
    fn glViewport(x: i32, y: i32, w: i32, h: i32);
    fn glMatrixMode(m: u32);
    fn glLoadIdentity();
    fn glLoadMatrixf(m: *const f32);
    fn glEnable(m: u32);
    fn glDisable(m: u32);
    fn glDepthFunc(m: u32);
    fn glDepthMask(m: u8);
    fn glBlendFunc(a: u32, b: u32);
    fn glBegin(m: u32);
    fn glEnd();
    fn glColor4ub(r: u8, g: u8, b: u8, a: u8);
    fn glVertex3f(x: f32, y: f32, z: f32);
    fn glVertex2f(x: f32, y: f32);
    fn glLineWidth(w: f32);
    fn glOrtho(l: f64, r: f64, b: f64, t: f64, n: f64, f: f64);
    fn glRasterPos2f(x: f32, y: f32);
    fn glListBase(b: u32);
    fn glCallLists(n: i32, t: u32, p: *const c_void);
    pub fn glGenLists(n: i32) -> u32;
    fn glReadPixels(x: i32, y: i32, w: i32, h: i32, format: u32, kind: u32, p: *mut c_void);
}
pub struct Camera {
    pub at: V,
    pub yaw: f32,
    pub pitch: f32,
}
impl Camera {
    pub fn project(&self, p: V, size: [i32; 2]) -> Option<[f32; 2]> {
        let (f, r, u) = self.basis();
        let d = sub(p, self.at);
        let z = dot(d, f);
        if z <= 0.08 {
            return None;
        }
        let tan = 30.0f32.to_radians().tan();
        let aspect = size[0] as f32 / size[1] as f32;
        Some([
            (dot(d, r) / (z * tan * aspect) + 1.0) * size[0] as f32 / 2.0,
            (1.0 - dot(d, u) / (z * tan)) * size[1] as f32 / 2.0,
        ])
    }
    pub fn home() -> Self {
        Self {
            at: [32.0, -42.0, 32.0],
            yaw: 2.22,
            pitch: -0.64,
        }
    }
    pub fn basis(&self) -> (V, V, V) {
        let (s, c) = self.yaw.sin_cos();
        let (p, q) = self.pitch.sin_cos();
        let f = [c * q, s * q, p];
        let r = [s, -c, 0.0];
        (f, r, cross(r, f))
    }
    pub fn ray(&self, p: [f32; 2], size: [i32; 2]) -> V {
        let (f, r, u) = self.basis();
        let t = (60.0f32.to_radians() / 2.0).tan();
        let x = (p[0] / size[0] as f32 * 2.0 - 1.0) * t * size[0] as f32 / size[1] as f32;
        let y = (1.0 - p[1] / size[1] as f32 * 2.0) * t;
        unit(add(f, add(mul(r, x), mul(u, y))))
    }
}
pub unsafe fn world(
    w: i32,
    h: i32,
    side: i32,
    cam: &Camera,
    scene: &Scene,
    selected: &[usize],
    ghost: &[Object],
    grid: f32,
    grid_origin: V,
    time: f32,
    animate: bool,
) {
    glViewport(side, 0, (w - side).max(1), h.max(1));
    glClearColor(0.18, 0.23, 0.30, 1.0);
    glClear(0x4100);
    glEnable(0x0B71);
    glDepthFunc(0x0203);
    glDisable(0x0DE1);
    glDisable(0x0B50);
    glDisable(0x0B44);
    let aspect = (w - side).max(1) as f32 / h.max(1) as f32;
    let f = 1.0 / (30.0f32.to_radians()).tan();
    let (n, z) = (0.08, 2500.0);
    let proj = [
        f / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        (z + n) / (n - z),
        -1.0,
        0.0,
        0.0,
        2.0 * z * n / (n - z),
        0.0,
    ];
    let (a, r, u) = cam.basis();
    let view = [
        r[0],
        u[0],
        -a[0],
        0.0,
        r[1],
        u[1],
        -a[1],
        0.0,
        r[2],
        u[2],
        -a[2],
        0.0,
        -dot(r, cam.at),
        -dot(u, cam.at),
        dot(a, cam.at),
        1.0,
    ];
    glMatrixMode(0x1701);
    glLoadMatrixf(proj.as_ptr());
    glMatrixMode(0x1700);
    glLoadMatrixf(view.as_ptr());
    for b in &scene.objects {
        object(b, 255);
    }
    if grid > 0.0 {
        let step = grid.max(if cam.at[2] > 15.0 { 2.0 } else { 0.5 });
        let n = 32;
        let (forward, _, _) = cam.basis();
        let t = if forward[2] < -0.001 {
            (grid_origin[2] - cam.at[2]) / forward[2]
        } else {
            0.0
        };
        let focus = add(cam.at, mul(forward, t.max(0.0).min(1000.0)));
        let center = [
            ((focus[0] - grid_origin[0]) / step).round() * step,
            ((focus[1] - grid_origin[1]) / step).round() * step,
        ];
        let extent = n as f32 * step;
        glColor4ub(90, 104, 120, 255);
        glBegin(1);
        for i in -n..=n {
            let a = i as f32 * step;
            for p in [
                [a + center[0], center[1] - extent, 0.012],
                [a + center[0], center[1] + extent, 0.012],
                [center[0] - extent, a + center[1], 0.012],
                [center[0] + extent, a + center[1], 0.012],
            ] {
                glVertex3f(
                    p[0] + grid_origin[0],
                    p[1] + grid_origin[1],
                    p[2] + grid_origin[2],
                );
            }
        }
        glEnd();
    }
    glLineWidth(2.0);
    for &i in selected {
        if let Some(b) = scene.objects.get(i) {
            outline(b);
        }
    }
    for b in ghost {
        glEnable(0x0BE2);
        glBlendFunc(0x0302, 0x0303);
        object(b, 135);
        outline(b);
        glDisable(0x0BE2);
    }
    // Spawn marker and an original procedural animation dummy give a human scale reference.
    let p = scene.spawn;
    glColor4ub(70, 231, 190, 255);
    glBegin(1);
    for i in 0..40 {
        let a = i as f32 * std::f32::consts::TAU / 40.0;
        let b = (i + 1) as f32 * std::f32::consts::TAU / 40.0;
        for t in [a, b] {
            glVertex3f(p[0] + t.cos() * 0.65, p[1] + t.sin() * 0.65, p[2] + 0.025);
        }
    }
    glEnd();
    mannequin(add(p, [2.0, 0.0, 0.0]), if animate { time } else { 0.0 });
    if let Some(&i) = selected.last() {
        if let Some(b) = scene.objects.get(i) {
            let a = b.at;
            glLineWidth(3.0);
            glBegin(1);
            for (k, c) in [[235, 99, 97], [98, 217, 144], [93, 167, 243]]
                .into_iter()
                .enumerate()
            {
                glColor4ub(c[0], c[1], c[2], 255);
                let mut end = a;
                end[k] += (dot(sub(a, cam.at), sub(a, cam.at)).sqrt() * 0.13).clamp(1.0, 7.0);
                glVertex3f(a[0], a[1], a[2] + 0.03);
                glVertex3f(end[0], end[1], end[2] + 0.03);
            }
            glEnd();
        }
    }
    glLineWidth(1.0);
}
unsafe fn object(b: &Object, alpha: u8) {
    glBegin(4);
    for t in b.triangles() {
        let n = unit(cross(sub(t[1], t[0]), sub(t[2], t[0])));
        let shade = 0.62 + 0.38 * dot(n, unit([-0.4, -0.6, 1.0])).max(0.0);
        let c = b.color.map(|x| (x as f32 * shade) as u8);
        glColor4ub(c[0], c[1], c[2], alpha);
        for p in t {
            glVertex3f(p[0], p[1], p[2]);
        }
    }
    glEnd();
}
pub unsafe fn edit_grid(b: &Object, preview: Option<&Object>) {
    let Some(grid) = &b.grid else {
        return;
    };
    glEnable(0x0BE2);
    glBlendFunc(0x0302, 0x0303);
    glDepthMask(0);
    glColor4ub(80, 225, 207, 160);
    glLineWidth(1.0);
    glBegin(1);
    for axis in 0..3 {
        let a = (axis + 1) % 3;
        let z = (axis + 2) % 3;
        for side in [0.0, 1.0] {
            for k in [a, z] {
                let other = if k == a { z } else { a };
                for i in 0..=grid.dims[k] {
                    let mut p = [0.0; 3];
                    p[axis] = if axis < 2 { -b.size[axis] / 2.0 } else { 0.0 }
                        + side * b.size[axis]
                        + if side == 0.0 { -0.003 } else { 0.003 };
                    p[k] = i as f32 / grid.dims[k] as f32 * b.size[k]
                        - if k < 2 { b.size[k] / 2.0 } else { 0.0 };
                    for end in [0.0, 1.0] {
                        p[other] =
                            end * b.size[other] - if other < 2 { b.size[other] / 2.0 } else { 0.0 };
                        let v = b.world(p);
                        glVertex3f(v[0], v[1], v[2]);
                    }
                }
            }
        }
    }
    glEnd();
    if let Some(p) = preview {
        object(p, 110);
        glLineWidth(2.0);
        outline(p);
    }
    glDepthMask(1);
    glDisable(0x0BE2);
    glLineWidth(1.0);
}
unsafe fn outline(b: &Object) {
    let [w, d, h] = b.size;
    let p = [
        [-w / 2.0, -d / 2.0, 0.0],
        [w / 2.0, -d / 2.0, 0.0],
        [w / 2.0, d / 2.0, 0.0],
        [-w / 2.0, d / 2.0, 0.0],
        [-w / 2.0, -d / 2.0, h],
        [w / 2.0, -d / 2.0, h],
        [w / 2.0, d / 2.0, h],
        [-w / 2.0, d / 2.0, h],
    ];
    glColor4ub(255, 210, 97, 255);
    glBegin(1);
    for [a, z] in [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ] {
        for i in [a, z] {
            let p = b.world(p[i]);
            glVertex3f(p[0], p[1], p[2] + 0.005);
        }
    }
    glEnd();
}
unsafe fn mannequin(p: V, time: f32) {
    let phase = (time * 3.0).sin() * 0.4;
    let segment = |a: V, b: V, width: f32, c: [u8; 3]| {
        let a = add(p, a);
        let b = add(p, b);
        let axis = unit(sub(b, a));
        let right = unit(cross(axis, [0.0, 1.0, 0.0]));
        let front = mul(unit(cross(axis, right)), width / 2.0);
        let right = mul(right, width / 2.0);
        let corners = |at: V| {
            [
                add(add(at, right), front),
                add(sub(at, right), front),
                sub(sub(at, right), front),
                add(sub(at, front), right),
            ]
        };
        let a = corners(a);
        let b = corners(b);
        glColor4ub(c[0], c[1], c[2], 255);
        glBegin(0x0007);
        for i in 0..4 {
            for v in [a[i], a[(i + 1) % 4], b[(i + 1) % 4], b[i]] {
                glVertex3f(v[0], v[1], v[2]);
            }
        }
        glEnd();
    };
    for s in [-1.0, 1.0] {
        let x = s * 0.14;
        let hip = [x, 0.0, 0.88];
        let knee = [x, phase * s * 0.4, 0.47];
        let foot = [x, -phase * s * 0.5, 0.08];
        segment(hip, knee, 0.18, [84, 151, 175]);
        segment(knee, foot, 0.15, [84, 151, 175]);
        let shoulder = [s * 0.29, 0.0, 1.4];
        let elbow = [s * 0.34, -phase * s * 0.5, 1.12];
        let hand = [s * 0.35, -phase * s * 0.6, 0.90];
        segment(shoulder, elbow, 0.14, [224, 180, 117]);
        segment(elbow, hand, 0.12, [224, 180, 117]);
    }
    object(
        &Object::new(
            "torso",
            Shape::Box,
            add(p, [0.0, 0.0, 0.90]),
            [0.44, 0.25, 0.57],
            [201, 157, 98],
        ),
        255,
    );
    object(
        &Object::new(
            "head",
            Shape::Sphere,
            add(p, [0.0, 0.0, 1.5]),
            [0.28, 0.27, 0.33],
            [221, 213, 195],
        ),
        255,
    );
}
pub unsafe fn ui_begin(w: i32, h: i32) {
    glViewport(0, 0, w, h);
    glDisable(0x0B71);
    glDisable(0x0DE1);
    glMatrixMode(0x1701);
    glLoadIdentity();
    glOrtho(0.0, w as f64, h as f64, 0.0, -1.0, 1.0);
    glMatrixMode(0x1700);
    glLoadIdentity();
}
pub unsafe fn rect(r: [f32; 4], c: [u8; 3]) {
    glColor4ub(c[0], c[1], c[2], 255);
    glBegin(7);
    for p in [[r[0], r[1]], [r[2], r[1]], [r[2], r[3]], [r[0], r[3]]] {
        glVertex2f(p[0], p[1]);
    }
    glEnd();
}
pub unsafe fn text(font: u32, x: f32, y: f32, s: &str, c: [u8; 3]) {
    glColor4ub(c[0], c[1], c[2], 255);
    glRasterPos2f(x, y + 14.0);
    glListBase(font - 32);
    let s: Vec<u8> = s
        .bytes()
        .map(|b| if (32..=126).contains(&b) { b } else { b'?' })
        .collect();
    glCallLists(s.len() as i32, 0x1401, s.as_ptr().cast());
}
pub unsafe fn capture(path: &std::path::Path, w: i32, h: i32) -> std::io::Result<()> {
    let mut p = vec![0u8; w as usize * h as usize * 4];
    glReadPixels(0, 0, w, h, 0x80E1, 0x1401, p.as_mut_ptr().cast());
    let mut b = vec![0u8; 54];
    b[..2].copy_from_slice(b"BM");
    b[2..6].copy_from_slice(&((54 + p.len()) as u32).to_le_bytes());
    b[10..14].copy_from_slice(&54u32.to_le_bytes());
    b[14..18].copy_from_slice(&40u32.to_le_bytes());
    b[18..22].copy_from_slice(&w.to_le_bytes());
    b[22..26].copy_from_slice(&h.to_le_bytes());
    b[26..28].copy_from_slice(&1u16.to_le_bytes());
    b[28..30].copy_from_slice(&32u16.to_le_bytes());
    b.extend(p);
    std::fs::write(path, b)
}
