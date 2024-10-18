use std::mem::zeroed;

use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::{ClientToScreen, ScreenToClient},
    UI::WindowsAndMessaging::*,
};

use super::util::*;

pub struct GameControl {
    pub hwnd: HWND,
    pub post: bool,
    pub w: u32,
    pub h: u32,
}

#[allow(non_snake_case, unused)]
impl GameControl {
    pub fn new(hwnd: HWND, post: bool) -> Self {
        // set_foreground_window(hwnd);
        let mut rect = unsafe { zeroed::<RECT>() };
        let _ = unsafe { GetClientRect(hwnd, &mut rect) };
        let mut point = unsafe { zeroed::<POINT>() };
        let _ = unsafe { ClientToScreen(hwnd, &mut point) };
        let w = (rect.right - rect.left) as u32;
        let h = (rect.bottom - rect.top) as u32;
        Self { hwnd, post, w, h }
    }

    pub fn isForeground(&self) -> bool {
        unsafe { GetForegroundWindow() == self.hwnd }
    }

    pub fn isNormalSize(&self) -> bool {
        if self.w / self.h == 16 / 9 {
            true
        } else {
            false
        }
    }

    pub fn SetFocus(&self) {
        set_foreground_window(self.hwnd);
    }

    pub fn toScreenPos(&self, x: i32, y: i32) -> (i32, i32) {
        let mut point = POINT { x, y };
        unsafe { ClientToScreen(self.hwnd, &mut point) };
        (
            point.x * self.w as i32 / 1600,
            point.y * self.h as i32 / 900,
        )
    }

    pub fn toScreenRect(&self, rect: (i32, i32, i32, i32)) -> (i32, i32, i32, i32) {
        let (x1, y1, x2, y2) = rect;
        let (x, y) = self.toScreenPos(x1, y1);
        let w = (x2 - x1) * self.w as i32 / 1600;
        let h = (y2 - y1) * self.h as i32 / 900;
        (x, y, x + w as i32, y + h as i32)
    }

    pub fn MouseMove(&self, x: i32, y: i32) {
        let (x, y) = self.toScreenPos(x, y);
        click(x, y, 0);
    }

    pub fn MouseGetPos(&self) -> (i32, i32) {
        let mut point = unsafe { zeroed::<POINT>() };
        unsafe { GetCursorPos(&mut point) };
        unsafe { ScreenToClient(self.hwnd, &mut point) };
        (point.x, point.y)
    }

    fn savePos(&self, restore: bool) {
        static mut MX: i32 = 0;
        static mut MY: i32 = 0;
        unsafe {
            if restore {
                click(MX, MY, 0);
            } else {
                let mut point = unsafe { zeroed::<POINT>() };
                GetCursorPos(&mut point);

                MX = point.x;
                MY = point.y;
            }
        }
    }

    pub fn SavePos(&self) {
        self.savePos(false);
    }

    pub fn RestorePos(&self) {
        self.savePos(true);
    }

    pub fn Wheel(&self, y: i32) {
        wheel(-y);
    }

    pub fn Click(&self, x: i32, y: i32) {
        if (self.post) {
            self.PostClick(x, y);
        } else {
            let (x, y) = self.toScreenPos(x, y);
            self.SetFocus();
            click(x, y, 1);
        }
    }

    /// 后台点击
    pub fn PostClick(&self, x: i32, y: i32) {
        // let (x, y) = self.toScreenPos(x, y);
        let mut point = POINT { x, y };
        unsafe {
            self.SavePos();
            self.MouseMove(x, y);
            PostMessageA(self.hwnd, WM_MOUSEMOVE, 0, &mut point as *mut _ as isize);
            PostMessageA(self.hwnd, WM_LBUTTONDOWN, 1, &mut point as *mut _ as isize);
            PostMessageA(self.hwnd, WM_LBUTTONUP, 1, &mut point as *mut _ as isize);
            self.Sleep(50);
            self.RestorePos();
        }
        // self.SavePos();
        // self.Click(x, y);
    }

    /// 后台点击 不移动鼠标
    pub fn PostClickLazy(&self, x: i32, y: i32) {
        // let (x, y) = self.toScreenPos(x, y);
        let mut point = POINT { x, y };
        unsafe {
            // self.SavePos();
            // self.MouseMove(x, y);
            PostMessageA(self.hwnd, WM_MOUSEMOVE, 0, &mut point as *mut _ as isize);
            PostMessageA(self.hwnd, WM_LBUTTONDOWN, 1, &mut point as *mut _ as isize);
            self.Sleep(20);
            PostMessageA(self.hwnd, WM_LBUTTONUP, 1, &mut point as *mut _ as isize);
            self.Sleep(20);
            // self.RestorePos();
        }
        // self.SavePos();
        // self.Click(x, y);
    }

    /// 完全相等的颜色
    pub fn EqColor(&self, x: i32, y: i32, rgb: u32) -> bool {
        hsl_sim(get_color(self.hwnd, x, y), rgb) == 0.
    }

    /// 基本相近的颜色.
    pub fn HuColor(&self, x: i32, y: i32, rgb: u32) -> bool {
        hsl_sim(get_color(self.hwnd, x, y), rgb) < 200.
    }

    /// 相近的颜色带参数
    pub fn HuColorM(&self, x: i32, y: i32, rgb: u32, m: f32) -> bool {
        hsl_sim(get_color(self.hwnd, x, y), rgb) < m
    }

    /// 基本相近的颜色 (批量)
    pub fn HuColorS(&self, x: i32, y: i32, rgb: u32) -> bool {
        hsl_sim(get_color_batch(x, y), rgb) < 100.
    }

    pub fn CheckColor(&self, x: i32, y: i32, color: &str) -> bool {
        // let (x, y) = self.toScreenPos(x, y);
        check_color_regex(self.hwnd, x, y, color)
    }

    pub fn SaveS(&self) {
        save_dc(self.hwnd);
    }

    pub fn FreeS(&self) {
        free_dc(self.hwnd);
    }

    pub fn CheckColorS(&self, x: i32, y: i32, color: &str) -> bool {
        // let (x, y) = self.toScreenPos(x, y);
        check_color_regex_batch(x, y, color)
    }

    pub fn GetColor(&self, x: i32, y: i32) -> String {
        // let (x, y) = self.toScreenPos(x, y);
        color_to_hex(get_color(self.hwnd, x, y))
    }

    pub fn GetColorS(&self, x: i32, y: i32) -> String {
        // let (x, y) = self.toScreenPos(x, y);
        color_to_hex(get_color_batch(x, y))
    }

    pub fn WaitEqColor(&self, x: i32, y: i32, rgb: u32, timeout: f64) -> bool {
        // let (x, y) = self.toScreenPos(x, y);
        let start = std::time::Instant::now();
        while start.elapsed().as_secs_f64() < timeout {
            if self.EqColor(x, y, rgb) {
                return true;
            }
            sleep(1);
        }
        false
    }

    pub fn WaitEqColor2(&self, x: i32, y: i32, rgb: u32, rgb2: u32, timeout: f64) -> bool {
        // let (x, y) = self.toScreenPos(x, y);
        let start = std::time::Instant::now();
        while start.elapsed().as_secs_f64() < timeout {
            if self.EqColor(x, y, rgb) || self.EqColor(x, y, rgb2) {
                return true;
            }
            sleep(1);
        }
        false
    }

    pub fn WaitHuColor(&self, x: i32, y: i32, rgb: u32, timeout: f64) -> bool {
        // let (x, y) = self.toScreenPos(x, y);
        let start = std::time::Instant::now();
        while start.elapsed().as_secs_f64() < timeout {
            if self.HuColor(x, y, rgb) {
                return true;
            }
            sleep(1);
        }
        false
    }

    pub fn WaitHuColor2(&self, x: i32, y: i32, rgb: u32, rgb2: u32, timeout: f64) -> bool {
        // let (x, y) = self.toScreenPos(x, y);
        let start = std::time::Instant::now();
        while start.elapsed().as_secs_f64() < timeout {
            if self.HuColor(x, y, rgb) || self.HuColor(x, y, rgb2) {
                return true;
            }
            sleep(1);
        }
        false
    }

    pub fn WaitColor(&self, x: i32, y: i32, color: &str, timeout: f64) -> bool {
        // let (x, y) = self.toScreenPos(x, y);
        let start = std::time::Instant::now();
        while start.elapsed().as_secs_f64() < timeout {
            if check_color_regex(self.hwnd, x, y, color) {
                return true;
            }
        }
        false
    }

    pub fn PressKey(&self, key: &str) {
        key_press(key, 0);
    }

    pub fn SendText(&self, text: &str) {
        // let old = get_clipboard_text();
        set_clipboard_text(text);
        combo_key_press(&["ctrl", "v"], 0);
        sleep(100);
        // set_clipboard_text(old.as_str());
    }

    pub fn Sleep(&self, ms: u32) {
        sleep(ms);
    }

    // pub fn PixelSearch(
    //     &self,
    //     rect: (i32, i32, i32, i32),
    //     rgb: u32,
    //     tolerance: f64,
    // ) -> Option<(i32, i32)> {
    //     let (x1, y1, x2, y2) = rect;
    //     let rect = self.toScreenRect(rect);
    //     let (offset_x, offset_y) = (rect.0 - x1, rect.1 - y1);
    //     let result = pixel_search(rect, rgb, tolerance);
    //     if let Some((x, y)) = result {
    //         Some((x - offset_x, y - offset_y))
    //     } else {
    //         None
    //     }
    // }

    // pub fn PixelSearchRev(
    //     &self,
    //     rect: (i32, i32, i32, i32),
    //     rgb: u32,
    //     tolerance: f64,
    // ) -> Option<(i32, i32)> {
    //     let (x1, y1, x2, y2) = rect;
    //     let rect = self.toScreenRect(rect);
    //     let (offset_x, offset_y) = (rect.0 - x1, rect.1 - y1);
    //     let result = pixel_search_reverse(rect, rgb, tolerance);
    //     if let Some((x, y)) = result {
    //         Some((x - offset_x, y - offset_y))
    //     } else {
    //         None
    //     }
    // }
}
