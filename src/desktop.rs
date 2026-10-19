use crate::shape_tools::{Brush, Edit};
use crate::{
    render::{self, Camera},
    scene::*,
};
use std::{
    ffi::c_void,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};
type H = *mut c_void;
#[repr(C)]
struct WndClass {
    size: u32,
    style: u32,
    proc_: extern "system" fn(H, u32, usize, isize) -> isize,
    cls: i32,
    wnd: i32,
    inst: H,
    icon: H,
    cursor: H,
    brush: H,
    menu: *const u16,
    name: *const u16,
    icon_sm: H,
}
#[repr(C)]
#[derive(Default)]
struct Msg {
    hwnd: usize,
    msg: u32,
    w: usize,
    l: isize,
    time: u32,
    x: i32,
    y: i32,
    private: u32,
}
#[repr(C)]
#[derive(Default)]
struct Rect {
    l: i32,
    t: i32,
    r: i32,
    b: i32,
}
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}
#[repr(C)]
struct Pfd {
    size: u16,
    version: u16,
    flags: u32,
    b: [u8; 20],
    masks: [u32; 3],
}
#[link(name = "user32")]
extern "system" {
    fn RegisterClassExW(c: *const WndClass) -> u16;
    fn CreateWindowExW(
        ex: u32,
        cls: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: H,
        menu: H,
        inst: H,
        p: H,
    ) -> H;
    fn DefWindowProcW(h: H, m: u32, w: usize, l: isize) -> isize;
    fn PeekMessageW(m: *mut Msg, h: H, a: u32, b: u32, r: u32) -> i32;
    fn TranslateMessage(m: *const Msg) -> i32;
    fn DispatchMessageW(m: *const Msg) -> isize;
    fn PostQuitMessage(c: i32);
    fn GetDC(h: H) -> H;
    fn ReleaseDC(h: H, dc: H) -> i32;
    fn DestroyWindow(h: H) -> i32;
    fn GetClientRect(h: H, r: *mut Rect) -> i32;
    fn GetAsyncKeyState(k: i32) -> i16;
    fn GetCursorPos(p: *mut Point) -> i32;
    fn ScreenToClient(h: H, p: *mut Point) -> i32;
    fn GetForegroundWindow() -> H;
    fn LoadCursorW(i: H, n: usize) -> H;
    fn SetWindowTextW(h: H, t: *const u16) -> i32;
    fn MessageBoxW(h: H, t: *const u16, c: *const u16, f: u32) -> i32;
    fn SetProcessDPIAware() -> i32;
    fn SetCapture(h: H) -> H;
    fn ReleaseCapture() -> i32;
    fn GetSystemMetrics(i: i32) -> i32;
}
#[link(name = "gdi32")]
extern "system" {
    fn ChoosePixelFormat(dc: H, p: *const Pfd) -> i32;
    fn SetPixelFormat(dc: H, f: i32, p: *const Pfd) -> i32;
    fn SwapBuffers(dc: H) -> i32;
    fn CreateFontW(
        h: i32,
        w: i32,
        e: i32,
        o: i32,
        weight: i32,
        italic: u32,
        under: u32,
        strike: u32,
        charset: u32,
        out: u32,
        clip: u32,
        quality: u32,
        pitch: u32,
        name: *const u16,
    ) -> H;
    fn SelectObject(dc: H, o: H) -> H;
    fn DeleteObject(o: H) -> i32;
}
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(n: *const u16) -> H;
}
#[link(name = "opengl32")]
extern "system" {
    fn wglCreateContext(dc: H) -> H;
    fn wglMakeCurrent(dc: H, rc: H) -> i32;
    fn wglDeleteContext(rc: H) -> i32;
    fn wglUseFontBitmapsW(dc: H, first: u32, count: u32, base: u32) -> i32;
    fn wglGetProcAddress(n: *const u8) -> *const c_void;
}
static QUIT: AtomicBool = AtomicBool::new(false);
static EVENTS: Mutex<Vec<Event>> = Mutex::new(Vec::new());
enum Event {
    Click([f32; 2]),
    Char(char),
    Wheel(f32),
}
extern "system" fn proc_(h: H, m: u32, w: usize, l: isize) -> isize {
    let ev = match m {
        0x201 => Some(Event::Click([
            (l as u16 as i16) as f32,
            ((l >> 16) as u16 as i16) as f32,
        ])),
        0x102 => char::from_u32(w as u32).map(Event::Char),
        0x20a => Some(Event::Wheel(((w >> 16) as u16 as i16) as f32 / 120.0)),
        _ => None,
    };
    if let Some(e) = ev {
        if let Ok(mut q) = EVENTS.lock() {
            q.push(e);
        }
        return 0;
    }
    if m == 0x10 {
        QUIT.store(true, Ordering::Relaxed);
        return 0;
    }
    if m == 2 {
        QUIT.store(true, Ordering::Relaxed);
        unsafe {
            PostQuitMessage(0);
        }
        return 0;
    }
    unsafe { DefWindowProcW(h, m, w, l) }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}
const SIDE: i32 = 324;
const GRIDS: [f32; 7] = [0.0, 0.1, 0.25, 0.5, 1.0, 2.0, 4.0];
include!("builder_controls.rs");
include!("build_files.rs");
include!("sculpt_controls.rs");
include!("object_actions.rs");
#[cfg(test)]
#[path = "desktop_tests.rs"]
mod tests;
#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Select,
    Place,
    Move,
    Rotate,
    Scale,
}
#[derive(Clone, Copy)]
enum Action {
    Combine,
    Subtract,
    ObjectMode(u8),
    Brush(Brush),
    Depth(i32),
    Radius(i32),
    Mirror(usize),
    Hollow,
    RoundEdges,
    Pad,
    Sculpt,
    Fill,
    SculptGrid,
    Through,
    Properties,
    Grab,
    Axis(usize),
    FreeAxis,
    ShowGrid,
    GridOrigin,
    Magnet,
    Kit(usize),
    Repeat,
    SavePrefab,
    LoadPrefab,
    Help,
    TopView,
    Tool(Tool),
    Shape(Shape),
    Color(usize),
    Save,
    Load,
    Undo,
    Redo,
    Home,
    Walk,
    Duplicate,
    Delete,
    Apply,
    Field(usize),
    Grid,
    RotateSnap,
    Collision,
    Lock,
    Spawn,
    Focus,
    Animate,
    Blank,
    Starter,
}
struct Hit {
    rect: [f32; 4],
    action: Action,
}
struct Drag {
    original: Scene,
    latched: bool,
    before: Scene,
    start: [f32; 2],
    point: V,
    pivot: V,
}
struct App {
    brush: Brush,
    depth: usize,
    brush_radius: usize,
    mirror: u8,
    extrude: bool,
    sculpting: bool,
    fill: bool,
    sculpt_resolution: usize,
    through: bool,
    carve: Option<Carve>,
    cut_preview: Option<Object>,
    inspector: bool,
    axis: Option<usize>,
    grid_visible: bool,
    grid_origin: V,
    magnet: bool,
    fine: bool,
    template: Vec<Object>,
    template_size: V,
    placement_yaw: f32,
    placement_height: f32,
    line_mode: bool,
    line_start: Option<V>,
    help: bool,
    pointer: [f32; 2],
    viewport: [i32; 2],
    scene: Scene,
    path: PathBuf,
    cam: Camera,
    selected: Vec<usize>,
    tool: Tool,
    shape: Shape,
    color: [u8; 3],
    size: V,
    grid: usize,
    angle: bool,
    undo: Vec<Scene>,
    redo: Vec<Scene>,
    status: String,
    fields: Vec<String>,
    active: Option<usize>,
    replace: bool,
    hits: Vec<Hit>,
    drag: Option<Drag>,
    ghost: Vec<Object>,
    walk: bool,
    feet: V,
    vz: f32,
    grounded: bool,
    animate: bool,
    scroll: f32,
    time: f32,
    dirty: bool,
}
impl App {
    fn new(path: PathBuf) -> Self {
        let (scene, status) = if path.exists() {
            match Scene::load(&path) {
                Ok(s) => (s, "Saved sandbox loaded".into()),
                Err(e) => (
                    Scene::starter(),
                    format!("Load failed: {e}. Starter shown; save needs a new filename."),
                ),
            }
        } else {
            (
                Scene::starter(),
                "Welcome. Choose a shape, then click PLACE and click the ground.".into(),
            )
        };
        let feet = scene.spawn;
        Self {
            brush: Brush::Rectangle,
            depth: 1,
            brush_radius: 1,
            mirror: 0,
            extrude: false,
            sculpting: false,
            fill: false,
            sculpt_resolution: 16,
            through: true,
            carve: None,
            cut_preview: None,
            inspector: false,
            axis: None,
            grid_visible: true,
            grid_origin: [0.0; 3],
            magnet: true,
            fine: false,
            template: vec![],
            template_size: [1.0; 3],
            placement_yaw: 0.0,
            placement_height: 0.0,
            line_mode: false,
            line_start: None,
            help: false,
            pointer: [800.0, 480.0],
            viewport: [1482, 942],
            scene,
            path,
            cam: Camera::home(),
            selected: vec![],
            tool: Tool::Select,
            shape: Shape::Box,
            color: COLORS[1],
            size: [2.0; 3],
            grid: 3,
            angle: true,
            undo: vec![],
            redo: vec![],
            status,
            fields: vec![String::new(); 13],
            active: None,
            replace: false,
            hits: vec![],
            drag: None,
            ghost: vec![],
            walk: false,
            feet,
            vz: 0.0,
            grounded: false,
            animate: true,
            scroll: 0.0,
            time: 0.0,
            dirty: false,
        }
    }
    fn snap(&self, x: f32) -> f32 {
        let g = if self.fine { 0.0 } else { GRIDS[self.grid] };
        if g == 0.0 {
            x
        } else {
            (x / g).round() * g
        }
    }
    fn checkpoint(&mut self, before: Scene) {
        if before == self.scene {
            return;
        }
        if self.undo.len() == 100 {
            self.undo.remove(0);
        }
        self.undo.push(before);
        self.redo.clear();
        self.dirty = true;
        self.sync_fields();
    }
    fn primary(&self) -> Option<&Object> {
        self.selected
            .last()
            .and_then(|&i| self.scene.objects.get(i))
    }
    fn sync_fields(&mut self) {
        if self.active.is_some() {
            return;
        }
        if let Some(b) = self.primary() {
            self.fields = vec![
                b.name.clone(),
                format!("{:.3}", b.at[0]),
                format!("{:.3}", b.at[1]),
                format!("{:.3}", b.at[2]),
                format!("{:.3}", b.size[0]),
                format!("{:.3}", b.size[1]),
                format!("{:.3}", b.size[2]),
                format!("{:.1}", b.yaw),
                b.color[0].to_string(),
                b.color[1].to_string(),
                b.color[2].to_string(),
                b.tilt[0].to_string(),
                b.tilt[1].to_string(),
            ];
        } else {
            self.fields = vec![
                "New object".into(),
                "0".into(),
                "0".into(),
                "0".into(),
                self.size[0].to_string(),
                self.size[1].to_string(),
                self.size[2].to_string(),
                "0".into(),
                self.color[0].to_string(),
                self.color[1].to_string(),
                self.color[2].to_string(),
                "0".into(),
                "0".into(),
            ];
        }
    }
    fn save(&mut self) {
        match self.scene.save(&self.path) {
            Ok(()) => {
                self.dirty = false;
                let recovery = self.path.with_file_name("recovery.grove");
                if recovery.exists() {
                    let _ = std::fs::remove_file(recovery);
                }
                self.status = "Saved worlds/sandbox.grove (previous version backed up)".into();
            }
            Err(e) => self.status = format!("Save failed: {e}"),
        }
    }
    fn apply(&mut self) {
        let parse = || -> Result<(V, V, f32, [u8; 3]), String> {
            let num = |i: usize| {
                self.fields[i]
                    .trim()
                    .parse::<f32>()
                    .map_err(|_| "Enter valid numbers".to_string())
            };
            let at = [num(1)?, num(2)?, num(3)?];
            let size = [num(4)?, num(5)?, num(6)?];
            let yaw = num(7)?;
            let rgb = |i: usize| {
                self.fields[i]
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| "RGB must be whole numbers from 0 to 255".to_string())
            };
            Ok((at, size, yaw, [rgb(8)?, rgb(9)?, rgb(10)?]))
        };
        match parse() {
            Ok((at, size, yaw, color)) => {
                let mut b = Object::new(&self.fields[0], self.shape, at, size, color);
                b.yaw = yaw;
                b.tilt = match (self.fields[11].parse(), self.fields[12].parse()) {
                    (Ok(x), Ok(y)) => [x, y],
                    _ => {
                        self.status = "Enter valid X/Y rotation angles".into();
                        return;
                    }
                };
                if !b.valid() {
                    self.status =
                        "Dimensions: 0.05 to 1000m. Positions: +/-10000m. Enter a name.".into();
                    return;
                }
                if let Some(&i) = self.selected.last() {
                    if self.scene.objects[i].locked {
                        self.status = "Unlock this object before editing".into();
                        return;
                    }
                    let before = self.scene.clone();
                    b.shape = self.scene.objects[i].shape;
                    b.grid = self.scene.objects[i].grid.clone();
                    b.solid = self.scene.objects[i].solid;
                    b.locked = false;
                    self.scene.objects[i] = b;
                    self.checkpoint(before);
                    self.status = "Object properties applied".into();
                } else {
                    self.size = size;
                    self.color = color;
                    self.status = "New object dimensions and color applied".into();
                }
            }
            Err(e) => self.status = e,
        }
    }
    fn action(&mut self, a: Action) {
        self.active = None;
        if self.carve.is_some() {
            self.end_cut(true);
        }
        if !matches!(
            a,
            Action::Sculpt
                | Action::Combine
                | Action::Subtract
                | Action::ObjectMode(_)
                | Action::Brush(_)
                | Action::Depth(_)
                | Action::Radius(_)
                | Action::Mirror(_)
                | Action::Hollow
                | Action::RoundEdges
                | Action::Pad
                | Action::Fill
                | Action::SculptGrid
                | Action::Through
                | Action::Help
                | Action::Properties
                | Action::Color(_)
                | Action::Save
        ) {
            self.sculpting = false;
        }
        if self.drag.is_some()
            && !matches!(
                a,
                Action::Grab
                    | Action::Tool(_)
                    | Action::Axis(_)
                    | Action::FreeAxis
                    | Action::Grid
                    | Action::RotateSnap
                    | Action::Magnet
                    | Action::ShowGrid
                    | Action::GridOrigin
                    | Action::Help
            )
        {
            self.finish_grab(false);
        }
        match a {
            Action::Tool(t) => {
                if matches!(t, Tool::Select | Tool::Place) {
                    self.finish_grab(false);
                }
                if self.drag.is_some() {
                    self.switch_grab(t);
                    return;
                }
                self.tool = t;
                self.status=match t{Tool::Place=>"Click a surface to place. Esc cancels. Right mouse + WASD moves the camera.",Tool::Select=>"Click to select. Ctrl + click adds to selection.",Tool::Move=>"Drag selected objects on the ground plane. Arrows/Page Up/Down nudge.",Tool::Rotate=>"Drag sideways to rotate selection around its last object.",Tool::Scale=>"Drag upward to enlarge selected objects. Inspector sets exact dimensions."}.into();
            }
            Action::Shape(s) => {
                self.template.clear();
                self.placement_yaw = 0.0;
                self.placement_height = 0.0;
                self.line_start = None;
                self.shape = s;
                self.tool = Tool::Place;
                self.selected.clear();
                self.size = match s {
                    Shape::Ramp => [3.0, 6.0, 2.0],
                    Shape::Steps => [3.0, 5.0, 2.0],
                    _ => [2.0; 3],
                };
                self.sync_fields();
                self.status = format!("{} ready: click a surface to place", s.name());
            }
            Action::Color(i) => {
                let before = self.scene.clone();
                self.color = COLORS[i];
                for &i in &self.selected {
                    if !self.scene.objects[i].locked {
                        self.scene.objects[i].color = self.color;
                    }
                }
                self.checkpoint(before);
                self.sync_fields();
            }
            Action::Save => self.save(),
            Action::Load => match Scene::load(&self.path) {
                Ok(s) => {
                    let before = std::mem::replace(&mut self.scene, s);
                    self.selected.clear();
                    self.checkpoint(before);
                    self.dirty = false;
                    self.status = "Sandbox reloaded. Undo can restore your previous scene.".into();
                }
                Err(e) => self.status = format!("Load failed: {e}"),
            },
            Action::Undo => {
                if let Some(s) = self.undo.pop() {
                    self.redo.push(std::mem::replace(&mut self.scene, s));
                    self.selected.clear();
                    self.dirty = true;
                    self.sync_fields();
                    self.status = "Edit undone".into();
                }
            }
            Action::Redo => {
                if let Some(s) = self.redo.pop() {
                    self.undo.push(std::mem::replace(&mut self.scene, s));
                    self.selected.clear();
                    self.dirty = true;
                    self.sync_fields();
                    self.status = "Edit restored".into();
                }
            }
            Action::Home => {
                self.cam = Camera::home();
                self.walk = false;
            }
            Action::Walk => {
                self.walk = !self.walk;
                self.active = None;
                if self.walk {
                    self.feet = self.scene.spawn;
                    self.vz = 0.0;
                    self.grounded = false;
                    self.cam.at = add(self.feet, [0.0, 0.0, 1.7]);
                    self.cam.yaw = std::f32::consts::FRAC_PI_2;
                    self.cam.pitch = 0.0;
                }
                self.status = if self.walk {
                    "WALK: hold right mouse to look, WASD moves, Space jumps. Tab returns to build."
                } else {
                    "Build mode: right mouse + WASD to fly"
                }
                .into();
            }
            Action::Duplicate => {
                let before = self.scene.clone();
                let ids = self.selected.clone();
                self.selected.clear();
                for i in ids {
                    let mut b = self.scene.objects[i].clone();
                    if b.locked {
                        continue;
                    }
                    b.name = format!("{} copy", b.name.chars().take(70).collect::<String>());
                    b.at = add(
                        b.at,
                        [self.snap(1.0).max(0.25), self.snap(1.0).max(0.25), 0.0],
                    );
                    self.selected.push(self.scene.objects.len());
                    self.scene.objects.push(b);
                }
                self.checkpoint(before);
            }
            Action::Delete => {
                let before = self.scene.clone();
                let ids = self.selected.clone();
                self.scene.objects = self
                    .scene
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(i, b)| !ids.contains(i) || b.locked)
                    .map(|(_, b)| b.clone())
                    .collect();
                self.selected.clear();
                self.checkpoint(before);
            }
            Action::Apply => self.apply(),
            Action::Field(i) => {
                self.active = Some(i);
                self.replace = true;
            }
            Action::Grid => {
                self.grid = (self.grid + 1) % GRIDS.len();
            }
            Action::RotateSnap => self.angle = !self.angle,
            Action::Collision | Action::Lock => {
                let before = self.scene.clone();
                for &i in &self.selected {
                    let b = &mut self.scene.objects[i];
                    if matches!(a, Action::Lock) {
                        b.locked = !b.locked;
                    } else if !b.locked {
                        b.solid = !b.solid;
                    }
                }
                self.checkpoint(before);
            }
            Action::Spawn => {
                let before = self.scene.clone();
                let p = if self.walk {
                    self.feet
                } else if let Some(b) = self.primary() {
                    [b.at[0], b.at[1], b.at[2] + b.size[2] + 0.02]
                } else {
                    let p = self.cam.at;
                    [
                        p[0],
                        p[1],
                        self.scene.ground(p, p[2] + 0.5).unwrap_or(0.0) + 0.02,
                    ]
                };
                self.scene.spawn = p;
                self.checkpoint(before);
                self.status = "Player spawn moved to the selected object or camera position".into();
            }
            Action::Focus => {
                if let Some(b) = self.primary() {
                    let at = add(b.at, [0.0, 0.0, b.size[2] * 0.5]);
                    let dist = b.size.iter().copied().fold(3.0, f32::max) * 1.8 + 3.0;
                    self.cam.at = add(at, [dist * 0.7, -dist, dist * 0.65]);
                    self.cam.yaw = 2.18;
                    self.cam.pitch = -0.48;
                    self.walk = false;
                }
            }
            Action::Animate => self.animate = !self.animate,
            Action::Blank | Action::Starter => {
                let before = self.scene.clone();
                self.scene = if matches!(a, Action::Starter) {
                    Scene::starter()
                } else {
                    let mut s = Scene::starter();
                    s.objects.truncate(1);
                    s
                };
                self.selected.clear();
                self.checkpoint(before);
                self.cam = Camera::home();
                self.walk = false;
                self.status =
                    "Scene created. Undo restores the previous scene; Save writes this one.".into();
            }
            _ => self.build_action(a),
        }
    }
    fn ray(&self, p: [f32; 2], w: i32, h: i32) -> V {
        self.cam
            .ray([p[0] - SIDE as f32, p[1]], [(w - SIDE).max(1), h.max(1)])
    }
    fn target(&self, d: V) -> Option<V> {
        if let Some((_, t)) = pick(&self.scene.objects, self.cam.at, d) {
            return Some(add(self.cam.at, mul(d, t)));
        }
        if d[2] < -0.0001 {
            let t = -self.cam.at[2] / d[2];
            if t > 0.0 && t < 2000.0 {
                return Some(add(self.cam.at, mul(d, t)));
            }
        }
        None
    }
    fn click(&mut self, p: [f32; 2], w: i32, h: i32, ctrl: bool) {
        if self.help && p[0] > SIDE as f32 && p[1] > 145.0 {
            self.help = false;
            return;
        }
        if let Some(a) = self
            .hits
            .iter()
            .rev()
            .find(|b| {
                p[0] >= b.rect[0] && p[0] <= b.rect[2] && p[1] >= b.rect[1] && p[1] <= b.rect[3]
            })
            .map(|b| b.action)
        {
            self.action(a);
            return;
        }
        if self.help {
            self.help = false;
            return;
        }
        self.active = None;
        if p[0] > SIDE as f32 && p[0] < (SIDE + 535) as f32 && (65.0..133.0).contains(&p[1]) {
            return;
        }
        if p[0] < SIDE as f32 || p[1] < 54.0 || p[1] > h as f32 - 52.0 || self.walk {
            return;
        }
        if self.drag.is_some() {
            self.finish_grab(false);
            return;
        }
        self.pointer = p;
        let d = self.ray(p, w, h);
        if self.sculpting {
            self.begin_cut(d);
            return;
        }
        if self.tool == Tool::Place {
            if let Some(at) = self.target(d) {
                self.stamp(at);
            }
            return;
        }
        if let Some((i, _)) = pick(&self.scene.objects, self.cam.at, d) {
            if ctrl {
                if let Some(j) = self.selected.iter().position(|x| *x == i) {
                    self.selected.remove(j);
                } else {
                    self.selected.push(i);
                }
            } else if !self.selected.contains(&i) {
                self.selected = vec![i];
            }
            self.sync_fields();
            if self.tool != Tool::Select && !ctrl {
                self.begin_grab(false);
                unsafe {
                    SetCapture(GetForegroundWindow());
                }
            }
        } else if !ctrl {
            self.selected.clear();
            self.sync_fields();
        }
    }
    fn drag_to(&mut self, p: [f32; 2], w: i32, h: i32) {
        let Some(dr) = &self.drag else {
            return;
        };
        let dx = p[0] - dr.start[0];
        let dy = p[1] - dr.start[1];
        let ray = self.ray(p, w, h);
        let mut dv = if let Some(k) = self.axis {
            let mut end = dr.pivot;
            end[k] += 1.0;
            let amount = match (self.screen(dr.pivot), self.screen(end)) {
                (Some(a), Some(b)) => {
                    let v = [b[0] - a[0], b[1] - a[1]];
                    (dx * v[0] + dy * v[1]) / (v[0] * v[0] + v[1] * v[1]).max(0.1)
                }
                _ => 0.0,
            };
            let mut v = [0.0; 3];
            v[k] = self.snap(amount);
            v
        } else if ray[2].abs() > 0.001 {
            let a = sub(
                add(
                    self.cam.at,
                    mul(ray, (dr.pivot[2] - self.cam.at[2]) / ray[2]),
                ),
                dr.point,
            );
            [self.snap(a[0]), self.snap(a[1]), 0.0]
        } else {
            [0.0; 3]
        };
        if self.tool == Tool::Move && self.magnet && !self.fine {
            let v: Vec<_> = self
                .selected
                .iter()
                .filter_map(|&i| {
                    let mut b = dr.before.objects[i].clone();
                    if b.locked {
                        return None;
                    }
                    b.at = add(b.at, dv);
                    Some(b)
                })
                .collect();
            dv = add(
                dv,
                edge_delta(
                    &self.scene.objects,
                    &v,
                    &self.selected,
                    self.axis,
                    GRIDS[self.grid].max(0.25) * 0.42,
                ),
            );
        }
        let degrees = dx * if self.fine { 0.1 } else { 0.5 };
        let turn = if self.angle && !self.fine {
            (degrees / 15.0).round() * 15.0
        } else {
            degrees
        };
        let scale = (-dy * if self.fine { 0.002 } else { 0.01 })
            .exp()
            .clamp(0.05, 20.0);
        for &i in &self.selected {
            let mut b = dr.before.objects[i].clone();
            if b.locked {
                continue;
            }
            match self.tool {
                Tool::Move => b.at = add(b.at, dv),
                Tool::Rotate => b.rotate_about(dr.pivot, self.axis.unwrap_or(2), turn),
                Tool::Scale => {
                    let mut offset = sub(b.at, dr.pivot);
                    for k in 0..3 {
                        if self.axis.is_none_or(|a| a == k) {
                            b.size[k] = (b.size[k] * scale).clamp(0.05, 1000.0);
                            offset[k] *= scale;
                        }
                    }
                    b.at = add(dr.pivot, offset);
                }
                _ => {}
            }
            if b.valid() {
                self.scene.objects[i] = b;
            }
        }
        self.sync_fields();
    }
    fn nudge(&mut self, d: V) {
        let before = self.scene.clone();
        let step = if self.fine {
            0.05
        } else {
            GRIDS[self.grid].max(0.1)
        };
        for &i in &self.selected {
            let b = &mut self.scene.objects[i];
            if !b.locked {
                let at = add(b.at, mul(d, step));
                if at.iter().all(|x| x.abs() <= 10000.0) {
                    b.at = at;
                }
            }
        }
        self.checkpoint(before);
    }
    fn walk_step(&mut self, dt: f32, dir: V, jump: bool) {
        let radius = 0.28;
        let move_axis = |feet: V, axis: usize, delta: f32, scene: &Scene| {
            let mut q = feet;
            q[axis] += delta;
            for b in scene.objects.iter().filter(|b| b.solid) {
                if let Some(grid) = &b.grid {
                    let mut blocked = false;
                    for z in [0.4, 0.9, 1.55] {
                        for (dx, dy) in [
                            (0.0, 0.0),
                            (-radius, 0.0),
                            (radius, 0.0),
                            (0.0, -radius),
                            (0.0, radius),
                        ] {
                            if grid.sample(b.local(add(q, [dx, dy, z])), b.size) {
                                blocked = true;
                            }
                        }
                    }
                    if blocked {
                        return feet;
                    }
                    continue;
                }
                let local = b.local(q);
                let inside = match b.shape {
                    Shape::Cylinder | Shape::Sphere => {
                        (local[0] / (b.size[0] / 2.0 + radius)).powi(2)
                            + (local[1] / (b.size[1] / 2.0 + radius)).powi(2)
                            < 1.0
                    }
                    _ => {
                        local[0].abs() < b.size[0] / 2.0 + radius
                            && local[1].abs() < b.size[1] / 2.0 + radius
                    }
                };
                if inside && b.at[2] < q[2] + 1.65 {
                    let mut surface = local;
                    // Test the near edge of a step rather than the entire staircase height.
                    surface[0] = surface[0].clamp(-b.size[0] * 0.49999, b.size[0] * 0.49999);
                    surface[1] = surface[1].clamp(-b.size[1] * 0.49999, b.size[1] * 0.49999);
                    let top = b.top(b.world(surface)).unwrap_or(b.at[2] + b.size[2]);
                    if top > q[2] + 0.36 && b.at[2] + b.size[2] > q[2] + 0.04 {
                        return feet;
                    }
                }
            }
            q
        };
        self.feet = move_axis(self.feet, 0, dir[0] * dt, &self.scene);
        self.feet = move_axis(self.feet, 1, dir[1] * dt, &self.scene);
        if jump && self.grounded {
            self.vz = 5.2;
            self.grounded = false;
        }
        self.vz -= 14.0 * dt;
        let old = self.feet[2];
        let mut next = old + self.vz * dt;
        if self.vz > 0.0 {
            let ceiling = self
                .scene
                .objects
                .iter()
                .filter(|b| b.solid)
                .flat_map(Object::triangles)
                .filter_map(|t| ray_triangle(self.feet, [0.0, 0.0, 1.0], t))
                .min_by(f32::total_cmp);
            if let Some(t) = ceiling {
                if t < 1.72 + (next - old) {
                    next = old;
                    self.vz = 0.0;
                }
            }
        }
        let g = self.scene.ground(self.feet, old + 0.36);
        self.grounded = false;
        if let Some(g) = g {
            if next <= g && self.vz <= 0.0 {
                next = g;
                self.vz = 0.0;
                self.grounded = true;
            }
        }
        self.feet[2] = next;
        if next < -30.0 {
            self.feet = self.scene.spawn;
            self.vz = 0.0;
        }
        self.cam.at = add(self.feet, [0.0, 0.0, 1.7]);
    }
}

unsafe fn button(app: &mut App, font: u32, r: [f32; 4], label: &str, a: Action, on: bool) {
    render::rect(r, if on { [40, 114, 133] } else { [39, 50, 65] });
    render::text(
        font,
        r[0] + 8.0,
        r[1] + 6.0,
        label,
        if on { [231, 255, 255] } else { [210, 221, 232] },
    );
    app.hits.push(Hit { rect: r, action: a });
}
unsafe fn label(font: u32, y: f32, s: &str) {
    render::text(font, 18.0, y, s, [132, 156, 177]);
}
include!("builder_ui.rs");
pub fn run() {
    unsafe {
        SetProcessDPIAware();
        let null = std::ptr::null_mut();
        let instance = GetModuleHandleW(std::ptr::null());
        let cls = wide("ProjectGroveSandbox");
        let wc = WndClass {
            size: std::mem::size_of::<WndClass>() as u32,
            style: 0x23,
            proc_,
            cls: 0,
            wnd: 0,
            inst: instance,
            icon: null,
            cursor: LoadCursorW(null, 32512),
            brush: null,
            menu: std::ptr::null(),
            name: cls.as_ptr(),
            icon_sm: null,
        };
        if RegisterClassExW(&wc) == 0 {
            return;
        }
        let sw = GetSystemMetrics(0);
        let sh = GetSystemMetrics(1);
        let hwnd = CreateWindowExW(
            0,
            cls.as_ptr(),
            wide("Project Grove - Sandbox World Builder").as_ptr(),
            0x10cf0000,
            40,
            30,
            (sw - 80).min(1500).max(800),
            (sh - 90).min(1000).max(600),
            null,
            null,
            instance,
            null,
        );
        if hwnd.is_null() {
            return;
        }
        let dc = GetDC(hwnd);
        let mut p = Pfd {
            size: std::mem::size_of::<Pfd>() as u16,
            version: 1,
            flags: 0x25,
            b: [0; 20],
            masks: [0; 3],
        };
        p.b[1] = 32;
        p.b[6] = 8;
        p.b[14] = 24;
        let pf = ChoosePixelFormat(dc, &p);
        let rc = if pf != 0 && SetPixelFormat(dc, pf, &p) != 0 {
            wglCreateContext(dc)
        } else {
            null
        };
        if rc.is_null() || wglMakeCurrent(dc, rc) == 0 {
            MessageBoxW(
                hwnd,
                wide("Could not start the OpenGL renderer.").as_ptr(),
                wide("Project Grove").as_ptr(),
                0x10,
            );
            return;
        }
        let vsync = wglGetProcAddress(b"wglSwapIntervalEXT\0".as_ptr());
        if !vsync.is_null() && (vsync as usize) > 3 && (vsync as isize) != -1 {
            let f: unsafe extern "system" fn(i32) -> i32 = std::mem::transmute(vsync);
            f(1);
        }
        let font = CreateFontW(
            -15,
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            1,
            0,
            0,
            4,
            0,
            wide("Segoe UI").as_ptr(),
        );
        let old_font = SelectObject(dc, font);
        let glyph = render::glGenLists(96);
        if glyph == 0 || wglUseFontBitmapsW(dc, 32, 96, glyph) == 0 {
            MessageBoxW(
                hwnd,
                wide("Could not create the interface font.").as_ptr(),
                wide("Project Grove").as_ptr(),
                0x10,
            );
            return;
        }
        let root = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .filter(|p| p.join("worlds").exists())
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        let path = root.join("worlds/sandbox.grove");
        let mut app = App::new(path);
        let recovery = root.join("worlds/recovery.grove");
        if recovery.exists() {
            let newer = std::fs::metadata(&recovery).and_then(|m| m.modified()).ok()
                > std::fs::metadata(&app.path).and_then(|m| m.modified()).ok();
            if newer {
                match Scene::load(&recovery) {
                    Ok(s) => {
                        app.scene = s;
                        app.dirty = true;
                        app.status =
                            "Recovered your previous unsaved session. Save to keep it.".into();
                    }
                    Err(e) => app.status = format!("Recovery could not load: {e}"),
                }
            }
        }
        app.sync_fields();
        if !app.path.exists() {
            app.save();
        }
        let mut previous = [false; 256];
        let mut mouse = Point::default();
        GetCursorPos(&mut mouse);
        let mut last = Instant::now();
        let mut count = 0;
        let mut fps_at = Instant::now();
        let demo=std::env::args().find_map(|x|x.strip_prefix("--feature-shot=").map(str::to_string));
        let capture_edit = demo.is_some()||std::env::args().any(|x| x == "--capture-edit");
        let verify = capture_edit || std::env::args().any(|x| x == "--capture");
        if capture_edit {
            app.scene = Scene::starter();
            app.scene.objects.truncate(1);
            let mut wall = Object::new(
                "Custom wall / real cut openings",
                Shape::Box,
                [0.0, 4.0, 0.0],
                [8.0, 0.75, 5.0],
                COLORS[1],
            );
            let mut grid = crate::object_grid::Grid::from_object(&wall, 16);
            grid.edit([6, 0, 0], [9, 1, 4], 1, true, false);
            grid.edit([1, 0, 5], [4, 1, 7], 1, true, false);
            grid.edit([11, 0, 5], [14, 1, 7], 1, true, false);
            wall.grid = Some(grid);
            app.scene.objects.push(wall);
            app.selected = vec![1];
            app.sculpting = true;
            app.cam.at = [10.0, -13.0, 9.0];
            app.cam.yaw = 2.1;
            app.cam.pitch = -0.35;
            app.sync_fields();
            app.status =
                "Object editing preview: cut doorway and windows, saved as real geometry".into();
            app.dirty = false;
        }
        let mut frame = 0;
        if let Some(kind)=&demo{
            let b=&mut app.scene.objects[1];b.size=[8.0,1.5,6.0];let mut g=crate::object_grid::Grid::from_object(b,32);
            let mut e=Edit{a:[11,0,0],b:[20,0,18],axis:1,side:-1,through:true,depth:3,fill:false,brush:Brush::Arch,mirror:0,radius:1,path:vec![]};
            if kind=="arches"{g.paint(&e);e.brush=Brush::Ellipse;e.a=[2,0,14];e.b=[8,0,20];e.mirror=1;g.paint(&e);app.brush=Brush::Arch;app.mirror=1;app.status="Arch doorway + mirrored round windows, cut with shape brushes".into();}
            else{e.brush=Brush::Rectangle;e.a=[8,0,7];e.b=[23,0,15];e.fill=true;b.grid=Some(g);b.extrude(&e).unwrap();g=b.grid.take().unwrap();app.extrude=true;app.fill=true;app.depth=3;app.status="Pull out: a selected region extends outward, connected to the original object".into();}
            b.grid=Some(g);app.sculpt_resolution=32;app.sync_fields();app.dirty=false;
        }
        while !QUIT.load(Ordering::Relaxed) {
            let mut msg = Msg::default();
            while PeekMessageW(&mut msg, null, 0, 0, 1) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let mut r = Rect::default();
            GetClientRect(hwnd, &mut r);
            let (w, h) = (r.r.max(SIDE + 100), r.b.max(100));
            let now = Instant::now();
            let dt = (now - last).as_secs_f32().min(0.04);
            last = now;
            let focused = GetForegroundWindow() == hwnd;
            let key = |k: usize| focused && GetAsyncKeyState(k as i32) < 0;
            let mut down = [false; 256];
            for k in 0..256 {
                down[k] = key(k);
            }
            let pressed = |k: usize| down[k] && !previous[k];
            let mut pos = Point::default();
            GetCursorPos(&mut pos);
            let delta = [(pos.x - mouse.x) as f32, (pos.y - mouse.y) as f32];
            mouse = pos;
            ScreenToClient(hwnd, &mut pos);
            let pixel = [pos.x as f32, pos.y as f32];
            app.viewport = [w, h];
            app.fine = down[0x10];
            if pixel[0] > SIDE as f32 && pixel[1] > 54.0 && pixel[1] < h as f32 - 52.0 {
                app.pointer = pixel;
            }
            if focused {
                let ev = EVENTS
                    .lock()
                    .map(|mut q| std::mem::take(&mut *q))
                    .unwrap_or_default();
                for e in ev {
                    match e {
                        Event::Click(p) => app.click(p, w, h, down[0x11]),
                        Event::Wheel(n) => {
                            if pixel[0] < SIDE as f32 {
                                app.scroll = (app.scroll - n * 72.0).max(0.0);
                            } else if !app.walk && app.active.is_none() {
                                app.scroll_world(n);
                            }
                        }
                        Event::Char(c) => {
                            if let Some(i) = app.active {
                                if c == '\r' {
                                    app.active = None;
                                    app.apply();
                                    app.sync_fields();
                                } else if c == '\u{8}' {
                                    if app.replace {
                                        app.fields[i].clear();
                                        app.replace = false;
                                    } else {
                                        app.fields[i].pop();
                                    }
                                } else if c.is_ascii()
                                    && !c.is_control()
                                    && c != '|'
                                    && app.fields[i].len() < 80
                                {
                                    if app.replace {
                                        app.fields[i].clear();
                                        app.replace = false;
                                    }
                                    app.fields[i].push(c);
                                }
                            }
                        }
                    }
                }
                if pressed(0x1b) {
                    app.finish_grab(true);
                    app.end_cut(true);
                    app.active = None;
                    app.tool = Tool::Select;
                    app.line_start = None;
                    app.sculpting = false;
                    app.ghost.clear();
                }
                if app.active.is_none() {
                    if pressed(0x70) {
                        app.action(Action::Help);
                    }
                    if pressed(9) {
                        app.action(Action::Walk);
                    }
                    if down[0x11] {
                        if pressed(0x53) {
                            app.action(Action::Save);
                        }
                        if pressed(0x5a) {
                            app.action(Action::Undo);
                        }
                        if pressed(0x59) {
                            app.action(Action::Redo);
                        }
                        if pressed(0x44) {
                            app.action(Action::Duplicate);
                            app.tool = Tool::Move;
                            app.begin_grab(true);
                        }
                    } else if !down[2] && !app.walk && !app.help {
                        if pressed(0x47) {
                            app.action(Action::Grab);
                        }
                        if pressed(0x52) && !app.sculpting {
                            app.cycle_mode();
                        }
                        if pressed(0x45) {
                            app.action(Action::Sculpt);
                        }
                        for (k, axis) in [(0x58, 0), (0x59, 1), (0x5a, 2)] {
                            if pressed(k) && !app.sculpting {
                                app.action(Action::Axis(axis));
                            }
                            if pressed(k) && app.sculpting {
                                app.action(Action::Mirror(axis));
                            }
                        }
                        if app.sculpting {
                            if pressed(0x43) {
                                app.action(Action::Fill);
                            }
                            if pressed(0x50) {
                                app.action(Action::ObjectMode(2));
                            }
                            if pressed(0x56) {
                                let index =
                                    Brush::ALL.iter().position(|b| *b == app.brush).unwrap();
                                app.action(Action::Brush(
                                    Brush::ALL[(index + 1) % Brush::ALL.len()],
                                ));
                            }
                        }
                        if pressed(0x42) {
                            app.action(Action::ShowGrid);
                        }
                        if pressed(0x4e) {
                            app.action(Action::Magnet);
                        }
                        if pressed(0xdd) {
                            if app.sculpting {
                                app.action(Action::Depth(1));
                            } else {
                                app.grid = (app.grid + 1) % GRIDS.len();
                            }
                        }
                        if pressed(0xdb) {
                            if app.sculpting {
                                app.action(Action::Depth(-1));
                            } else {
                                app.grid = (app.grid + GRIDS.len() - 1) % GRIDS.len();
                            }
                        }
                        for (k, t) in [
                            (0x31, Tool::Select),
                            (0x32, Tool::Place),
                            (0x33, Tool::Move),
                            (0x34, Tool::Rotate),
                            (0x35, Tool::Scale),
                        ] {
                            if pressed(k) {
                                app.action(Action::Tool(t));
                            }
                        }
                        if pressed(0x46) {
                            app.action(Action::Focus);
                        }
                        if pressed(0x2e) {
                            app.action(Action::Delete);
                        }
                        if pressed(0x0d) {
                            if app.drag.is_some() {
                                app.finish_grab(false);
                            } else if app.inspector {
                                app.apply();
                            }
                        }
                        if app.drag.is_none() && !app.sculpting {
                            for (k, d) in [
                                (0x25, [-1.0, 0.0, 0.0]),
                                (0x27, [1.0, 0.0, 0.0]),
                                (0x26, [0.0, 1.0, 0.0]),
                                (0x28, [0.0, -1.0, 0.0]),
                                (0x21, [0.0, 0.0, 1.0]),
                                (0x22, [0.0, 0.0, -1.0]),
                            ] {
                                if pressed(k) {
                                    app.nudge(d);
                                }
                            }
                        }
                    }
                }
                if app.carve.is_some() {
                    if down[1] {
                        app.update_cut(pixel);
                    } else {
                        app.end_cut(false);
                    }
                }
                if app.drag.is_some() {
                    if app.drag.as_ref().is_some_and(|d| d.latched) || down[1] {
                        if pixel[0] > SIDE as f32 && !down[2] && !app.help {
                            app.drag_to(pixel, w, h);
                        }
                    } else {
                        app.finish_grab(false);
                    }
                }
                if app.active.is_none() && app.carve.is_none() && (app.drag.is_none() || down[2]) {
                    let look = down[2] && pixel[0] > SIDE as f32 && pixel[1] > 53.0;
                    if look {
                        app.cam.yaw -= delta[0] * 0.004;
                        app.cam.pitch = (app.cam.pitch - delta[1] * 0.004).clamp(-1.52, 1.52);
                    }
                    if app.walk {
                        let (s, c) = app.cam.yaw.sin_cos();
                        let mut dir = [0.0; 3];
                        let speed = if down[0x10] { 6.0 } else { 3.0 };
                        for (k, v) in [
                            (0x57, [c, s, 0.0]),
                            (0x53, [-c, -s, 0.0]),
                            (0x44, [s, -c, 0.0]),
                            (0x41, [-s, c, 0.0]),
                        ] {
                            if down[k] {
                                dir = add(dir, v);
                            }
                        }
                        if dot(dir, dir) > 0.0 {
                            dir = mul(unit(dir), speed);
                        }
                        let steps = (dt / 0.008).ceil().max(1.0) as u32;
                        for step in 0..steps {
                            app.walk_step(dt / steps as f32, dir, step == 0 && pressed(0x20));
                        }
                    } else if look {
                        let (f, right, _) = app.cam.basis();
                        let mut dir = [0.0; 3];
                        for (k, v) in [
                            (0x57, f),
                            (0x53, mul(f, -1.0)),
                            (0x44, right),
                            (0x41, mul(right, -1.0)),
                            (0x45, [0.0, 0.0, 1.0]),
                            (0x51, [0.0, 0.0, -1.0]),
                        ] {
                            if down[k] {
                                dir = add(dir, v);
                            }
                        }
                        app.cam.at = add(
                            app.cam.at,
                            mul(dir, dt * if down[0x10] { 30.0 } else { 10.0 }),
                        );
                        if app.drag.is_some() {
                            app.switch_grab(app.tool);
                        }
                    }
                }
            } else {
                app.end_cut(true);
                if let Some(d) = app.drag.take() {
                    app.scene = d.original;
                    ReleaseCapture();
                }
                if let Ok(mut q) = EVENTS.lock() {
                    q.clear();
                }
            }
            app.ghost.clear();
            if !app.walk
                && app.tool == Tool::Place
                && pixel[0] > SIDE as f32
                && pixel[1] > 53.0
                && pixel[1] < h as f32 - 46.0
            {
                let ray = app.ray(pixel, w, h);
                if let Some(p) = app.target(ray) {
                    app.ghost = app.placement(p);
                }
            }
            if app.animate {
                app.time += dt;
            }
            render::world(
                w,
                h,
                SIDE,
                &app.cam,
                &app.scene,
                &app.selected,
                &app.ghost,
                if app.grid_visible {
                    GRIDS[app.grid].max(0.1)
                } else {
                    0.0
                },
                app.grid_origin,
                app.time,
                app.animate,
            );
            if app.sculpting {
                if let Some(b) = app.primary() {
                    render::edit_grid(b, app.cut_preview.as_ref());
                }
            }
            ui(&mut app, glyph, w, h);
            if pressed(0x7b) || (verify && frame == 30) {
                let p = if let Some(kind)=&demo{root.join(format!("feature-{kind}.bmp"))}else{root.join(if capture_edit {
                    "sculpt-preview.bmp"
                } else {
                    "sandbox-preview.bmp"
                })};
                app.status = match render::capture(&p, w, h) {
                    Ok(()) => "Screenshot saved: sandbox-preview.bmp".into(),
                    Err(e) => format!("Screenshot failed: {e}"),
                };
            }
            SwapBuffers(dc);
            frame += 1;
            if verify && frame >= 32 {
                QUIT.store(true, Ordering::Relaxed);
            }
            previous = down;
            count += 1;
            if fps_at.elapsed().as_secs_f32() > 1.0 {
                let fps = count as f32 / fps_at.elapsed().as_secs_f32();
                SetWindowTextW(
                    hwnd,
                    wide(&format!(
                        "Project Grove - Sandbox World Builder | {:.0} FPS | {} objects{}",
                        fps,
                        app.scene.objects.len(),
                        if app.dirty { " *" } else { "" }
                    ))
                    .as_ptr(),
                );
                fps_at = Instant::now();
                count = 0;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        app.finish_grab(false);
        app.end_cut(true);
        if app.dirty && !verify {
            let recovery = root.join("worlds/recovery.grove");
            if let Err(e) = app.scene.save(&recovery) {
                MessageBoxW(
                    hwnd,
                    wide(&format!("Could not write recovery save: {e}")).as_ptr(),
                    wide("Project Grove").as_ptr(),
                    0x10,
                );
            }
        }
        SelectObject(dc, old_font);
        DeleteObject(font);
        wglMakeCurrent(null, null);
        wglDeleteContext(rc);
        ReleaseDC(hwnd, dc);
        DestroyWindow(hwnd);
    }
}
