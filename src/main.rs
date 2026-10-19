#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod building_kit;
#[cfg(windows)]
mod desktop;
mod object_grid;
#[cfg(windows)]
mod render;
mod scene;
mod shape_tools;
mod paint;
fn main() {
    #[cfg(windows)]
    desktop::run();
    #[cfg(not(windows))]
    eprintln!("The Grove desktop sandbox currently requires Windows.");
}
