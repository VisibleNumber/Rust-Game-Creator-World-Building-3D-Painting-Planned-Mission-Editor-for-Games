#[repr(C)]
struct OpenFile {
    size: u32,
    owner: H,
    instance: H,
    filter: *const u16,
    custom: *mut u16,
    custom_count: u32,
    index: u32,
    file: *mut u16,
    file_count: u32,
    file_title: *mut u16,
    title_count: u32,
    directory: *const u16,
    title: *const u16,
    flags: u32,
    offset: u16,
    extension: u16,
    default_extension: *const u16,
    data: isize,
    hook: H,
    template: *const u16,
    reserved: H,
    reserved_word: u32,
    flags_ex: u32,
}
#[link(name = "comdlg32")]
extern "system" {
    fn GetOpenFileNameW(f: *mut OpenFile) -> i32;
    fn GetSaveFileNameW(f: *mut OpenFile) -> i32;
}
fn choose_build(save: bool, directory: &std::path::Path) -> Option<PathBuf> {
    let mut buffer = vec![0u16; 4096];
    if save {
        let name = wide("my-building.grove");
        buffer[..name.len()].copy_from_slice(&name);
    }
    let filter = wide("Grove buildings (*.grove)\0*.grove\0");
    let dir = wide(&directory.to_string_lossy());
    let title = wide(if save {
        "Save your building or object"
    } else {
        "Choose a building or object to place"
    });
    let ext = wide("grove");
    unsafe {
        let mut f: OpenFile = std::mem::zeroed();
        f.size = std::mem::size_of::<OpenFile>() as u32;
        f.owner = GetForegroundWindow();
        f.filter = filter.as_ptr();
        f.index = 1;
        f.file = buffer.as_mut_ptr();
        f.file_count = buffer.len() as u32;
        f.directory = dir.as_ptr();
        f.title = title.as_ptr();
        f.default_extension = ext.as_ptr();
        f.flags = 0x80000 | 0x800 | 8 | if save { 2 } else { 0x1000 };
        let result = if save {
            GetSaveFileNameW(&mut f)
        } else {
            GetOpenFileNameW(&mut f)
        };
        if result == 0 {
            return None;
        }
    }
    let len = buffer.iter().position(|x| *x == 0).unwrap_or(buffer.len());
    Some(PathBuf::from(String::from_utf16_lossy(&buffer[..len])))
}
