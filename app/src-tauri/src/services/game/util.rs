use std::collections::HashMap;
use std::sync::Mutex;
use std::{ffi::c_void, mem::size_of, ptr};

// use crossbeam_channel::{unbounded, Receiver, Sender};
// use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::{
        DataExchange::*,
        Diagnostics::{Debug::*, ToolHelp::*},
        Memory::*,
        ProcessStatus::*,
        SystemServices::IMAGE_DOS_HEADER,
        Threading::*,
    },
    UI::{Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
};

#[derive(thiserror::Error, Debug)]
pub enum Win32Error {
    #[error("sys error:`{0}`")]
    Sys(#[from] windows_core::Error),
    #[error("io error:`{0}`")]
    Io(#[from] std::io::Error),
    #[error("custom error:`{0}`")]
    Custom(String),
}

unsafe fn cwstr(p_text: *const u16) -> String {
    let len = (0..).take_while(|&i| *p_text.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(p_text, len);
    let text = String::from_utf16_lossy(slice);
    text
}

unsafe fn cstr(p_text: *const u8) -> String {
    let len = (0..).take_while(|&i| *p_text.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(p_text, len);
    let text = String::from_utf8_lossy(slice);
    text.to_string()
}

pub fn str_to_pcwstr(s: &str) -> *mut u16 {
    widestring::U16CString::from_str(s).unwrap().into_raw()
}

pub(crate) fn set_foreground_window(hwnd: HWND) -> bool {
    unsafe {
        // 显示窗口并激活
        ShowWindow(hwnd, SW_RESTORE);
        let result = SetForegroundWindow(hwnd);
        result == TRUE
    }
}

/// 鼠标移动 (绝对坐标)
pub fn mouse_move_to(x: i32, y: i32) {
    unsafe {
        // 移动鼠标
        mouse_event(
            MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
            (x << 16) / GetSystemMetrics(SM_CXSCREEN),
            (y << 16) / GetSystemMetrics(SM_CYSCREEN),
            0,
            0,
        )
    };
}

pub fn mouse_down() {
    unsafe { mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0) }
}

pub fn mouse_up() {
    unsafe { mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0) }
}
// 左键单击
pub fn click(x: i32, y: i32, count: u32) {
    mouse_move_to(x, y);
    for _ in 0..count {
        mouse_down();
        sleep(1);
        mouse_up();
    }
}

pub fn wheel(delta: i32) {
    unsafe { mouse_event(MOUSEEVENTF_WHEEL, 0, 0, delta as i32, 0) }
    sleep(1);
}

pub fn sleep(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

// pub async fn sleep_async(ms: u32) -> tokio::time::Sleep {
//     tokio::time::sleep(std::time::Duration::from_millis(ms as u64))
// }
// pub fn sleepf(s: f64) {
//     std::thread::sleep(std::time::Duration::from_secs_f64(s));
// }

/// 设置剪贴板文本
pub fn set_clipboard_text(text: &str) {
    unsafe {
        if OpenClipboard(0) == 0 {
            println!("[set_clipboard_text] OpenClipboard failed");
            return;
        }
        EmptyClipboard();
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let wlen = (wide.len() * std::mem::size_of::<u16>()) as usize;
        let h_global = GlobalAlloc(GMEM_MOVEABLE, wlen);
        if !h_global.is_null() {
            let p_global = GlobalLock(h_global);
            if !p_global.is_null() {
                let p_text = p_global as *mut u16;
                std::ptr::copy_nonoverlapping(wide.as_ptr(), p_text, wide.len());
                if SetClipboardData(13 /*CF_UNICODETEXT */, h_global as isize) == 0 {
                    println!("[set_clipboard_text] SetClipboardData failed");
                    GlobalFree(h_global);
                }
                GlobalUnlock(h_global);
            } else {
                println!("[set_clipboard_text] GlobalLock failed");
                GlobalFree(h_global);
            }
        } else {
            println!("[set_clipboard_text] GlobalAlloc failed");
        }
        CloseClipboard();
    }
}

// pub fn get_clipboard_text() -> String {
//     unsafe {
//         OpenClipboard(0);
//         let h_global = GetClipboardData(13 /*CF_UNICODETEXT */) as *mut c_void;
//         if !h_global.is_null() {
//             let p_global = GlobalLock(h_global);
//             let p_text = p_global as *const u16;
//             let len = (0..).take_while(|&i| *p_text.offset(i) != 0).count();
//             let slice = std::slice::from_raw_parts(p_text, len);
//             let text = String::from_utf16_lossy(slice);
//             GlobalUnlock(h_global);
//             CloseClipboard();
//             text
//         } else {
//             println!("[get_clipboard_text] GetClipboardData failed");
//             CloseClipboard();
//             String::default()
//         }
//     }
// }

fn key_down(key: u16) {
    unsafe { keybd_event(key as u8, 0, 0, 0) }
}

fn key_up(key: u16) {
    unsafe { keybd_event(key as u8, 0, KEYEVENTF_KEYUP, 0) }
}

pub fn key_press(key: &str, duration: u32) {
    let vkey = key_to_vkey(key);
    if vkey == 0 {
        panic!("无效的按键: {}", key);
    }
    key_down(vkey);
    if duration > 0 {
        sleep(duration);
    }
    key_up(vkey);
}

/// 组合键
pub fn combo_key_press(keys: &[&str], duration: u32) {
    let ve = keys
        .iter()
        .map(|key| key_to_vkey(key))
        .filter(|key| *key != 0)
        .collect::<Vec<u16>>();
    let vkeys = ve.as_slice();
    for &vkey in vkeys {
        key_down(vkey);
    }
    if duration > 0 {
        sleep(duration);
    }
    for &vkey in vkeys.iter().rev() {
        key_up(vkey);
    }
}

fn key_to_vkey(key: &str) -> u16 {
    match key {
        "a" => 0x41,
        "b" => 0x42,
        "c" => 0x43,
        "d" => 0x44,
        "e" => 0x45,
        "f" => 0x46,
        "g" => 0x47,
        "h" => 0x48,
        "i" => 0x49,
        "j" => 0x4A,
        "k" => 0x4B,
        "l" => 0x4C,
        "m" => 0x4D,
        "n" => 0x4E,
        "o" => 0x4F,
        "p" => 0x50,
        "q" => 0x51,
        "r" => 0x52,
        "s" => 0x53,
        "t" => 0x54,
        "u" => 0x55,
        "v" => 0x56,
        "w" => 0x57,
        "x" => 0x58,
        "y" => 0x59,
        "z" => 0x5A,
        "0" => 0x30,
        "1" => 0x31,
        "2" => 0x32,
        "3" => 0x33,
        "4" => 0x34,
        "5" => 0x35,
        "6" => 0x36,
        "7" => 0x37,
        "8" => 0x38,
        "9" => 0x39,
        "space" => 0x20,
        "enter" => 0x0D,
        "backspace" => 0x08,
        "esc" => 0x1B,
        "escape" => 0x1B,
        "left" => 0x25,
        "up" => 0x26,
        "right" => 0x27,
        "down" => 0x28,
        "shift" => 0xA0,
        "lshift" => 0xA0,
        "rshift" => 0xA1,
        "ctrl" => 0xA2,
        "lctrl" => 0xA2,
        "rctrl" => 0xA3,
        "alt" => 0xA4,
        "lalt" => 0xA4,
        "ralt" => 0xA5,
        "tab" => 0x09,
        "capslock" => 0x14,
        "numlock" => 0x90,
        "scrolllock" => 0x91,
        "printscreen" => 0x2C,
        "insert" => 0x2D,
        "del" => 0x2E,
        "delete" => 0x2E,
        "home" => 0x24,
        "end" => 0x23,
        "pageup" => 0x21,
        "pagedown" => 0x22,
        "f1" => 0x70,
        "f2" => 0x71,
        "f3" => 0x72,
        "f4" => 0x73,
        "f5" => 0x74,
        "f6" => 0x75,
        "f7" => 0x76,
        "f8" => 0x77,
        "f9" => 0x78,
        "f10" => 0x79,
        "f11" => 0x7A,
        "f12" => 0x7B,
        "lwin" => 0x5B,
        "rwin" => 0x5C,
        "apps" => 0x5D,
        "media_next_track" => 0xB0,
        "media_prev_track" => 0xB1,
        "media_play_pause" => 0xB3,
        "media_stop" => 0xB2,
        "volume_mute" => 0xAD,
        "volume_down" => 0xAE,
        "volume_up" => 0xAF,
        "media_select" => 0xB5,
        "browser_back" => 0xA6,
        "browser_forward" => 0xA7,
        "browser_refresh" => 0xA8,
        "browser_stop" => 0xA9,
        "browser_search" => 0xAA,
        "browser_favorites" => 0xAB,
        "browser_home" => 0xAC,
        "launch_mail" => 0xB4,
        "launch_media_select" => 0xB6,
        "launch_app1" => 0xB7,
        "launch_app2" => 0xB8,
        _ => 0,
    }
}

pub(crate) fn get_process_by_name(name: &str) -> Result<u32, Win32Error> {
    unsafe {
        let mut processes: Vec<PROCESSENTRY32W> = Vec::new();
        let snapshot_handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

        if snapshot_handle == INVALID_HANDLE_VALUE {
            return Err(Win32Error::Custom("INVALID_HANDLE_VALUE".to_string()));
        }
        let mut process_entry = std::mem::zeroed::<PROCESSENTRY32W>();
        process_entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let rst = Process32FirstW(snapshot_handle, &mut process_entry);
        if rst == FALSE {
            CloseHandle(snapshot_handle);
            return Err(Win32Error::Sys(windows_core::Error::from_win32()));
        }

        processes.push(process_entry);

        while Process32NextW(snapshot_handle, &mut process_entry) == TRUE {
            processes.push(process_entry);
        }

        CloseHandle(snapshot_handle);

        for process in processes {
            let sz_exe_file = cwstr(&process.szExeFile as *const u16);
            if sz_exe_file == name {
                return Ok(process.th32ProcessID);
            }
        }
    }
    return Ok(0);
}

pub(crate) fn get_window_by_process_name(name: &str) -> Option<HWND> {
    if let Ok(pid) = get_process_by_name(name) {
        if let Some(hwnd) = get_window_by_process(pid) {
            return Some(hwnd);
        }
    }
    None
}

pub(crate) fn kill_process(pid: u32) -> Result<bool, Win32Error> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle == -1 {
            return Err(Win32Error::Sys(windows_core::Error::from_win32()));
        }
        if TerminateProcess(handle, 0) == FALSE {
            return Err(Win32Error::Sys(windows_core::Error::from_win32()));
        }
        CloseHandle(handle);
        Ok(true)
    }
}

pub fn shell_execute(
    lp_file: &str,
    lp_parameters: Option<&str>,
    lp_directory: Option<&str>,
) -> Result<SHELLEXECUTEINFOW, Win32Error> {
    unsafe {
        let file = str_to_pcwstr(lp_file);
        let parameters = lp_parameters.map_or(ptr::null(), |p| str_to_pcwstr(p));
        let directory = lp_directory.map_or(ptr::null(), |d| str_to_pcwstr(d));
        let mut pc: SHELLEXECUTEINFOW = std::mem::zeroed();
        pc.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        pc.lpVerb = str_to_pcwstr("runas");
        pc.lpFile = file;
        pc.lpDirectory = directory;
        pc.lpParameters = parameters;
        pc.nShow = SW_SHOWNORMAL;
        pc.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_NOCLOSEPROCESS;
        ShellExecuteExW(&mut pc);
        // let _ = CloseHandle(pc.hProcess);
        // loop {
        // println!("ShellExecuteW: {}", pc.hProcess);
        //     sleep(Duration::from_millis(500));
        //     if pc.hProcess.0 != 0 {
        //         break;
        //     }
        // }
        Ok(pc)
    }
}

pub(crate) fn get_module_by_name(
    hprocess: HANDLE,
    module_name: &str,
) -> Result<MODULEENTRY32, Win32Error> {
    unsafe {
        let mut h_module = std::mem::zeroed::<MODULEENTRY32>();

        'out: for _ in 0..50 {
            let mut modules: Vec<HMODULE> = Vec::with_capacity(1024);
            modules.resize(1024, HMODULE::default());
            let mut cb_needed = 0;

            if TRUE
                == EnumProcessModules(
                    hprocess,
                    modules.as_mut_ptr(),
                    (modules.len() * size_of::<HMODULE>()) as u32,
                    &mut cb_needed,
                )
            {
                modules.resize(
                    cb_needed as usize / size_of::<HMODULE>(),
                    HMODULE::default(),
                );
                for it in modules {
                    let mut sz_module_name = [0u8; MAX_PATH as usize];
                    if GetModuleBaseNameA(hprocess, it, sz_module_name.as_mut_ptr(), MAX_PATH) == 0
                    {
                        continue;
                    }
                    if module_name != cstr(sz_module_name.as_ptr()) {
                        continue;
                    }
                    let mut mod_info = std::mem::zeroed::<MODULEINFO>();
                    if GetModuleInformation(
                        hprocess,
                        it,
                        &mut mod_info,
                        size_of::<MODULEINFO>() as u32,
                    ) == FALSE
                    {
                        continue;
                    }

                    h_module.modBaseAddr = mod_info.lpBaseOfDll as *mut u8;
                    h_module.modBaseSize = mod_info.SizeOfImage;
                    break 'out;
                }
            }

            sleep(200);
        }
        Ok(h_module)
    }
}

pub(crate) fn get_memory_by_pattern(
    hprocess: HANDLE,
    lpbaseaddress: *const c_void,
    dwsize: usize,
    pattern: &str,
    offset: usize,
) -> Result<*const c_void, Win32Error> {
    unsafe {
        let up = VirtualAlloc(
            ptr::null_mut(),
            dwsize,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if up.is_null() {
            return Err(Win32Error::Sys(windows_core::Error::from_win32()));
        }

        // 把整个模块读出来
        ReadProcessMemory(hprocess, lpbaseaddress, up, dwsize, ptr::null_mut());
        let address = pattern_scan(up, pattern); // ver 3.7 - last
        if address.is_null() {
            return Err(Win32Error::Custom("outdated pattern".to_string()));
        }

        println!("Pattern found at: {:p}", address);

        // 计算相对地址 (FPS)
        let pfps = {
            let mut rip = address as usize;
            println!("RIP: {:x}", rip);
            rip = rip.wrapping_add(3);
            println!("RIP: {:x} {}", rip, *(rip as *const i32));
            rip = rip.wrapping_add(std::ptr::read_unaligned(rip as *const i32) as usize + 6);
            println!("RIP: {:x} {}", rip, *(rip as *const i32));
            rip = rip.wrapping_add(std::ptr::read_unaligned(rip as *const i32) as usize + 4);
            println!("RIP: {:x} {:p}", rip, up);
            rip - up as usize + offset
        } as *const c_void;
        println!("pfps: {:p}", pfps);

        VirtualFree(up, 0, MEM_RELEASE);

        return Ok(pfps);
    }
}

lazy_static! {
    static ref DC_CACHE: Mutex<HDC> = Mutex::new(HDC::default());
}

pub fn save_dc(hwnd: HWND) {
    unsafe {
        // If the function succeeds, the return value is a handle to the DC for the specified window's client area.
        let hdc = GetDC(hwnd);
        let mut dc_cache = DC_CACHE.lock().unwrap();
        ReleaseDC(hwnd, *dc_cache);
        *dc_cache = hdc;
    }
}

pub fn free_dc(hwnd: HWND) {
    unsafe {
        let mut dc_cache = DC_CACHE.lock().unwrap();
        if !(*dc_cache) == 0 {
            ReleaseDC(hwnd, *dc_cache);
        }
        *dc_cache = HDC::default();
    }
}

pub fn get_color(hwnd: HWND, x: i32, y: i32) -> u32 {
    unsafe {
        let hdc = GetDC(hwnd);
        let ret = GetPixel(hdc, x, y);
        ReleaseDC(hwnd, hdc);
        ret
    }
}

pub fn get_color_batch(x: i32, y: i32) -> u32 {
    unsafe {
        let dc_cache = DC_CACHE.lock().unwrap();
        let ret = GetPixel(*dc_cache, x, y);
        ret
    }
}

pub fn check_color_regex(hwnd: HWND, x: i32, y: i32, pattern: &str) -> bool {
    let pixel = get_color(hwnd, x, y);
    check_color_raw(pixel, pattern)
}

pub fn check_color_regex_batch(x: i32, y: i32, pattern: &str) -> bool {
    let pixel = get_color_batch(x, y);
    check_color_raw(pixel, pattern)
}

pub fn color_to_hex(color: u32) -> String {
    let r = color & 0xFF;
    let g = color >> 8 & 0xFF;
    let b = color >> 16 & 0xFF;

    format!("{:02X}{:02X}{:02X}", r, g, b)
}

fn check_color_raw(pixel: u32, pattern: &str) -> bool {
    let color = color_to_hex(pixel);

    if color.len() == pattern.len() {
        let color = color.chars().collect::<Vec<char>>();
        let pattern = pattern.chars().collect::<Vec<char>>();
        for i in 0..color.len() {
            if color[i] != pattern[i] && pattern[i] != '.' {
                return false;
            }
        }
        return true;
    }
    is_match(&color, pattern)
}

pub fn is_match(s: &str, pattern: &str) -> bool {
    // 缓存编译好的正则表达式
    lazy_static! {
        static ref RE_CACHE: Mutex<HashMap<String, Regex>> = Mutex::new(HashMap::new());
    }

    let mut cache = RE_CACHE.lock().unwrap();
    if !cache.contains_key(pattern) {
        cache.insert(pattern.to_string(), Regex::new(pattern).unwrap());
    }
    let re = cache.get(pattern).unwrap();
    re.is_match(s)
}

pub(crate) fn write_memory_until_exit(hprocess: HANDLE, pfps: *const c_void, target_fps: isize) {
    unsafe {
        let mut dw_exit_code = STILL_ACTIVE as u32;
        while dw_exit_code == STILL_ACTIVE as u32 {
            let _ = GetExitCodeProcess(hprocess, &mut dw_exit_code);
            sleep(2000);

            let mut fps = 0isize;
            if FALSE
                == ReadProcessMemory(
                    hprocess,
                    pfps,
                    &mut fps as *mut isize as *mut c_void,
                    size_of::<usize>(),
                    ptr::null_mut(),
                )
            {
                CloseHandle(hprocess);
                eprintln!(
                    "ReadProcessMemory error: {}",
                    windows_core::Error::from_win32()
                );
                return;
            }
            if fps <= 0 {
                continue;
            }
            if fps != target_fps {
                WriteProcessMemory(
                    hprocess,
                    pfps,
                    &target_fps as *const isize as *const c_void,
                    size_of::<isize>(),
                    ptr::null_mut(),
                );
                // println!("FPS: {}", fps);
            }
        }
        // exit
        CloseHandle(hprocess);
    }
}

fn pattern_to_byte(pattern: &str) -> Vec<i16> {
    let mut bytes = Vec::new();
    for chunk in pattern.split_whitespace() {
        if chunk == "??" {
            bytes.push(-1); // 将 "??" 转换为 0
        } else {
            match i16::from_str_radix(chunk, 16) {
                Ok(byte) => bytes.push(byte),
                Err(_) => {
                    eprintln!("Invalid hex byte: {}", chunk);
                    return Vec::new();
                }
            }
        }
    }
    bytes
}

unsafe fn pattern_scan(module: *const c_void, signature: &str) -> *const c_void {
    let dos_header = module.cast::<IMAGE_DOS_HEADER>();
    let nt_headers = module
        .add((*dos_header).e_lfanew as usize)
        .cast::<IMAGE_NT_HEADERS64>();

    let size_of_image = (*nt_headers).OptionalHeader.SizeOfImage;
    let pattern_bytes = pattern_to_byte(signature);
    let scan_bytes = module;

    let s = pattern_bytes.len();
    let d = pattern_bytes.as_slice();

    for i in 0..size_of_image as usize - s {
        let mut found = true;
        for j in 0..s {
            if *(scan_bytes.add(i + j).cast::<u8>()) != d[j] as u8 && *d.get_unchecked(j) != -1i16 {
                found = false;
                break;
            }
        }
        if found {
            return scan_bytes.add(i);
        }
    }
    std::ptr::null()
}

pub(crate) fn move_window(hwnd: HWND, x: i32, y: i32) {
    unsafe {
        SetWindowPos(hwnd, 0, x, y, 0, 0, SWP_NOSIZE);
    }
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, wi: LPARAM) -> BOOL {
    let pid = (*(wi as *mut EnumWindowsData)).pid;
    let mut process_id = 0u32;
    let _ = GetWindowThreadProcessId(hwnd, &mut process_id);

    if pid == process_id && IsWindowVisible(hwnd) == 1 {
        // 如果找到匹配的进程 ID，返回 FALSE 停止枚举
        (*(wi as *mut EnumWindowsData)).hwnd = hwnd;
        SetLastError(0); // 重置错误代码
        0 // FALSE
    } else {
        // 继续枚举
        1 // TRUE
    }
}

#[repr(C)]
pub struct EnumWindowsData {
    pub hwnd: HWND,
    pub pid: u32,
}

pub(crate) fn get_window_by_process(pid: u32) -> Option<HWND> {
    unsafe {
        let mut wi = std::mem::zeroed::<EnumWindowsData>();
        wi.pid = pid;
        let _ = EnumWindows(Some(enum_windows_proc), &mut wi as *const _ as LPARAM);

        if wi.hwnd != 0 {
            // 检查是否找到窗口
            Some(wi.hwnd)
        } else {
            None
        }
    }
}
pub(crate) fn get_window_rect(hwnd: HWND) -> Option<RECT> {
    unsafe {
        let mut rect = std::mem::zeroed::<RECT>();
        if GetWindowRect(hwnd, &mut rect) == 1 {
            Some(rect)
        } else {
            None
        }
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub enum HotKeyState {
//     /// The [`HotKey`] is pressed (the key is down).
//     Pressed,
//     /// The [`HotKey`] is released (the key is up).
//     Released,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub struct GlobalHotKeyEvent {
//     /// Id of the associated [`HotKey`].
//     pub id: u32,
//     /// State of the associated [`HotKey`].
//     pub state: HotKeyState,
// }

// pub type GlobalHotKeyEventReceiver = Receiver<GlobalHotKeyEvent>;
// type GlobalHotKeyEventHandler = Box<dyn Fn(GlobalHotKeyEvent) + Send + Sync + 'static>;

// static GLOBAL_HOTKEY_CHANNEL: Lazy<(Sender<GlobalHotKeyEvent>, GlobalHotKeyEventReceiver)> =
//     Lazy::new(unbounded);
// static GLOBAL_HOTKEY_EVENT_HANDLER: OnceCell<Option<GlobalHotKeyEventHandler>> = OnceCell::new();

// impl GlobalHotKeyEvent {
//     /// Returns the id of the associated [`HotKey`].
//     pub fn id(&self) -> u32 {
//         self.id
//     }
//     /// Returns the state of the associated [`HotKey`].

//     pub fn state(&self) -> HotKeyState {
//         self.state
//     }

//     /// Gets a reference to the event channel's [`GlobalHotKeyEventReceiver`]
//     /// which can be used to listen for global hotkey events.
//     ///
//     /// ## Note
//     ///
//     /// This will not receive any events if [`GlobalHotKeyEvent::set_event_handler`] has been called with a `Some` value.
//     pub fn receiver<'a>() -> &'a GlobalHotKeyEventReceiver {
//         &GLOBAL_HOTKEY_CHANNEL.1
//     }

//     /// Set a handler to be called for new events. Useful for implementing custom event sender.
//     ///
//     /// ## Note
//     ///
//     /// Calling this function with a `Some` value,
//     /// will not send new events to the channel associated with [`GlobalHotKeyEvent::receiver`]
//     pub fn set_event_handler<F: Fn(GlobalHotKeyEvent) + Send + Sync + 'static>(f: Option<F>) {
//         if let Some(f) = f {
//             let _ = GLOBAL_HOTKEY_EVENT_HANDLER.set(Some(Box::new(f)));
//         } else {
//             let _ = GLOBAL_HOTKEY_EVENT_HANDLER.set(None);
//         }
//     }

//     pub(crate) fn send(event: GlobalHotKeyEvent) {
//         if let Some(handler) = GLOBAL_HOTKEY_EVENT_HANDLER.get_or_init(|| None) {
//             handler(event);
//         } else {
//             let _ = GLOBAL_HOTKEY_CHANNEL.0.send(event);
//         }
//     }
// }

// pub(crate) fn set_keyboard_hook(proc: KeyboardProc) -> HHOOK {
//     unsafe { SetWindowsHookExA(WH_KEYBOARD_LL, Some(proc), 0, 0) }
// }

// pub(crate) fn del_keyboard_hook(hookid: HHOOK) -> bool {
//     unsafe { UnhookWindowsHookEx(hookid) != 0 }
// }
// // 钩子函数原型
// type KeyboardProc = unsafe extern "system" fn(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> isize;
// // 钩子函数
// pub unsafe extern "system" fn keyboard_hook(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> isize {
//     if ncode >= 0 {
//         // 这里可以访问 KBDLLHOOKSTRUCT 结构体中的键盘信息
//         let hook_struct = lparam as *const KBDLLHOOKSTRUCT;
//         // 例如，获取虚拟键码
//         let vk_code = (*hook_struct).vkCode;
//         let scan_code = (*hook_struct).scanCode;
//         let flags = (*hook_struct).flags;
//         let extra_info = (*hook_struct).dwExtraInfo;
//         println!(
//             "Virtual Key Code: {}, Scan Code: {}, Flags: {}, Extra Info: {}",
//             vk_code, scan_code, flags, extra_info
//         );
//         // 这里可以处理键盘事件
//     }
//     CallNextHookEx(0, ncode, wparam, lparam)
// }
