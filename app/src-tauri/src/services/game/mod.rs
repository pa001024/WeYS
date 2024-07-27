use std::{ffi::c_void, time::Duration};

use serde::{Deserialize, Serialize};
use slider::cap_slide;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Emitter, Runtime,
};
use winreg::{enums::*, RegKey, RegValue};

use self::util::*;
use game::GameControl;
mod game;
mod slider;
mod util;

const GAME_PROCESS: &str = "YuanShen.exe";

#[derive(Serialize, Deserialize, Clone)]
struct LoginPayload {
    id: String,
    success: bool,
}

// 自动切世界权限
#[tauri::command]
async fn auto_open(state: i32, post: bool) -> bool {
    if let Some(hwnd) = get_window_by_process_name(GAME_PROCESS) {
        let interval = Duration::from_millis(100);
        let ctl = GameControl::new(hwnd, post);
        if !post {
            ctl.SetFocus();
        }
        if post {
            ctl.PostClick(469, 840); // 世界权限
        } else {
            ctl.Click(469, 840); // 世界权限
        }
        tokio::time::sleep(interval).await;
        match state {
            1 => ctl.Click(485, 735), // 直接加入
            2 => ctl.Click(473, 785), // 确认后可加入
            3 => ctl.Click(457, 686), // 无法加入
            _ => {}
        }

        true
    } else {
        false
    }
}

#[tauri::command]
async fn auto_join(uid: String) -> bool {
    let pid = get_process_by_name(GAME_PROCESS).unwrap_or(0);
    let mut elapsed = Duration::from_secs(0);
    let timeout = Duration::from_secs(2);
    let interval = Duration::from_millis(100);
    let interval2 = Duration::from_millis(10);
    if let Some(hwnd) = get_window_by_process(pid) {
        let ctl = GameControl::new(hwnd, false);
        ctl.SetFocus();
        if !ctl.isNormalSize() {
            set_clipboard_text(uid.as_str());
            return false;
        }
        while elapsed <= timeout {
            if enter_f2(&ctl) {
                // UID搜索结果不唯一
                while ctl.HuColor(1332, 257, 0x333333) {
                    ctl.Click(82, 179);
                    ctl.Wheel(1);
                    tokio::time::sleep(interval2).await;
                }
                println!("粘贴UID");
                while elapsed <= timeout {
                    if ctl.EqColor(1332, 361, 0xFFFFFF) {
                        set_clipboard_text(uid.as_str());
                        tokio::time::sleep(interval2).await;
                        elapsed += interval2;
                        ctl.Click(1249, 102); // 粘贴
                    } else {
                        break;
                    }
                    tokio::time::sleep(interval2).await;
                    elapsed += interval2;
                }
                println!("开始搜索");
                while elapsed <= timeout {
                    // 搜索
                    ctl.Click(1428, 102);
                    tokio::time::sleep(interval).await;
                    // 点进入
                    ctl.Click(1411, 203);
                    tokio::time::sleep(interval).await;
                    elapsed += interval * 2;
                }
                return true;
            }
            tokio::time::sleep(interval).await;
            elapsed += interval;
        }
    }
    return false;
}

fn enter_f2(ctl: &GameControl) -> bool {
    save_dc(ctl.hwnd);
    // F2 界面
    if ctl.CheckColorS(63, 38, "D3BC8E") && ctl.CheckColorS(1537, 54, "ECE5D8") {
        free_dc(ctl.hwnd);
        return true;
    }
    // 加载界面
    if ctl.CheckColorS(333, 333, "1C1C22|FFFFFF|000000") {
        free_dc(ctl.hwnd);
        return false;
    }
    // 1P主界面
    if ctl.CheckColorS(298, 44, "FFFFFF") {
        free_dc(ctl.hwnd);
        ctl.PressKey("f2");
        return false;
    }
    false
}

#[tauri::command]
async fn launch_game<R: Runtime>(
    app: AppHandle<R>,
    id: String,
    path: String,
    cmds: String,
    unlock: bool,
) -> bool {
    let pid = get_process_by_name(GAME_PROCESS).unwrap_or(0);
    let mut x = 0;
    let mut y = 0;
    if pid > 0 {
        if let Some(hwnd) = get_window_by_process(pid) {
            if let Some(rect) = get_window_rect(hwnd) {
                x = rect.left;
                y = rect.top;
            }
        }
        let _ = kill_process(pid);
    }
    let pi = shell_execute(path.as_str(), Some(cmds.as_str()), None);
    if let Err(err) = pi {
        println!("Failed to launch game: {:?}", err);
        return false;
    }
    let pi = pi.unwrap();
    println!("Process ID: {}", pi.hProcess);

    // 游戏进程启动
    let _ = app.emit(
        "game_init",
        LoginPayload {
            id: id.clone(),
            success: true,
        },
    );

    // 帧率解锁
    if unlock {
        unlockfps(pi.hProcess);
    }

    // 游戏窗口设置
    let now = std::time::Instant::now();
    let timeout = Duration::from_secs(30);
    while now.elapsed() < timeout {
        if let Some(hwnd) = get_window_by_process_name(GAME_PROCESS) {
            if pid > 0 {
                move_window(hwnd, x, y);
                if let Some(rect) = get_window_rect(hwnd) {
                    if rect.left == x && rect.top == y {
                        println!(
                            "window_rect hwnd = {}, pos = {},{}",
                            hwnd, rect.left, rect.top
                        );
                        let _ = app.emit(
                            "game_start",
                            LoginPayload {
                                id: id.clone(),
                                success: true,
                            },
                        );
                        return true;
                    }
                }
            } else {
                if let Some(rect) = get_window_rect(hwnd) {
                    if rect.right - rect.left > 500 {
                        let _ = app.emit(
                            "game_start",
                            LoginPayload {
                                id: id.clone(),
                                success: true,
                            },
                        );
                        return true;
                    }
                }
            }
        }
        sleep(100);
    }
    let _ = app.emit(
        "game_start",
        LoginPayload {
            id: id.clone(),
            success: false,
        },
    );

    true
}

fn unlockfps(pid: isize) -> bool {
    match get_module_by_name(pid, "UnityPlayer.dll") {
        Ok(h_unity_player) => {
            match get_memory_by_pattern(
                pid,
                h_unity_player.modBaseAddr as *const c_void,
                h_unity_player.modBaseSize as usize,
                "7F 0E E8 ?? ?? ?? ?? 66 0F 6E C8",
                h_unity_player.modBaseAddr as usize,
            ) {
                Ok(pfps) => {
                    println!("FPS Offset: {:?}", pfps);
                    write_memory_until_exit(pid, pfps, 140);
                    return true;
                }
                Err(err) => println!("Failed to get memory by pattern: {:?}", err),
            }
        }
        Err(err) => println!("Failed to get UnityPlayer.dll: {:?}", err),
    }
    false
}

#[tauri::command]
async fn auto_login<R: Runtime>(app: AppHandle<R>, id: String, login: String, pwd: String) {
    if let Some(hwnd) = get_window_by_process_name(GAME_PROCESS) {
        let ctl = GameControl::new(hwnd, false);
        // 适龄提示
        println!("自动登录");
        if !ctl.WaitEqColor2(1510, 70, 0x148BCE, 0x0A4566, 30.) {
            println!("登录超时");
            let _ = app.emit(
                "game_login",
                LoginPayload {
                    id: id.clone(),
                    success: false,
                },
            );
            return;
        }
        println!("登录开始");
        // 判断登陆框
        if !ctl.EqColor(1510, 70, 0x148BCE) {
            println!("需输入密码");
            let _ = app.emit(
                "game_input",
                LoginPayload {
                    id: id.clone(),
                    success: true,
                },
            );
            ctl.SetFocus();
            ctl.Sleep(100);
            ctl.WaitEqColor(626, 274, 0xFFFFFF, 5.); // 登陆框
            ctl.Click(971, 348);
            ctl.Sleep(100);
            ctl.SendText(login.as_str());
            ctl.Sleep(300);
            ctl.Click(994, 420);
            ctl.Sleep(100);
            ctl.SendText(pwd.as_str());
            ctl.Sleep(300);
            if !ctl.EqColor(580, 505, 0xDEBC60) {
                ctl.Click(581, 514);
                ctl.Sleep(100);
            }
            ctl.Click(973, 580);
            sleep(200);
        }
        let now = std::time::Instant::now();
        let timeout = Duration::from_secs(30);
        while now.elapsed() < timeout {
            // 循环判断登录成功
            if ctl.WaitEqColor(1510, 70, 0x148BCE, 1.) {
                break;
            }
            // 登陆框灰色 说明有滑块
            if ctl.WaitEqColor(603, 250, 0x7E7E7E, 1.) && ctl.EqColor(667, 271, 0xFFFFFF) {
                // 点击此处重试
                if ctl.EqColor(855, 503, 0x8A9DCA) || ctl.EqColor(855, 503, 0xA0B1D9) {
                    ctl.Click(869, 504);
                    sleep(1000);
                }
                // 判断滑块框
                if ctl.WaitEqColor(888, 514, 0xDFE1E2, 1.) {
                    sleep(300);
                    auto_slide(hwnd);
                }
            }

            ctl.Click(973, 580);
        }
        if now.elapsed() >= timeout {
            let _ = app.emit("game_enter", LoginPayload { id, success: false });
            return;
        }
        let _ = app.emit(
            "game_ready",
            LoginPayload {
                id: id.clone(),
                success: true,
            },
        );
        // 点击进入
        let rst_before = get_uid();
        if rst_before.uid.is_empty() {
            println!("等待加载(注册表)");
            let now = std::time::Instant::now();
            let timeout = Duration::from_secs(30);

            while now.elapsed() < timeout {
                if ctl.EqColor(761, 827, 0xFFFFFF) {
                    ctl.Click(819, 838);
                } else if !ctl.EqColor(677, 297, 0xFFFFFF) {
                    ctl.PostClickLazy(819, 838);
                }
                let rst = get_uid();
                sleep(200);
                if !rst.uid.is_empty() {
                    println!("成功加载");
                    let _ = app.emit("game_enter", LoginPayload { id, success: true });
                    return;
                }
            }
        } else {
            if ctl.WaitEqColor(761, 827, 0xFFFFFF, 20.) {
                ctl.PostClick(819, 838);
                // 等待加载完毕
                println!("等待加载");
                let now = std::time::Instant::now();
                let timeout = Duration::from_secs(30);
                while now.elapsed() < timeout {
                    ctl.Click(819, 838);
                    let rst = get_uid();
                    sleep(200);
                    if !rst.uid.is_empty() {
                        println!("成功加载");
                        let _ = app.emit("game_enter", LoginPayload { id, success: true });
                        return;
                    }
                }
            }
        }
        let _ = app.emit("game_enter", LoginPayload { id, success: false });
    }
}

#[tauri::command]
async fn auto_setup<R: Runtime>(app: AppHandle<R>, id: String, autosend: bool, post: bool) {
    if let Some(hwnd) = get_window_by_process_name(GAME_PROCESS) {
        let ctl = GameControl::new(hwnd, post);
        ctl.WaitHuColor(72, 33, 0xE9C48F, 40.);
        // ctl.focus();
        ctl.Sleep(100);
        let _ = app.emit(
            "game_login",
            LoginPayload {
                id: id.clone(),
                success: true,
            },
        );
        if ctl.WaitEqColor(77, 49, 0xFFFFFF, 2.) // 主界面
                            && ctl.EqColor(385, 58, 0xFFFFFF)
        // F2为白色
        {
            println!("成功加载");
            ctl.Click(385, 58); // 点击F2
            if ctl.WaitEqColor(469, 840, 0xECE5D8, 2.) {
                println!("设置权限");
                ctl.Click(469, 840); // 世界权限
                ctl.Sleep(100);
                if autosend {
                    ctl.Click(457, 686); // 无法加入
                } else {
                    ctl.Click(485, 735); // 直接加入
                }
                ctl.Sleep(100);
                ctl.Click(469, 840); // 世界权限
                println!("登录结束");

                return;
            }
        } else {
            println!("未加载成功");
        }
    }
}

#[tauri::command]
async fn kill_game() -> bool {
    let pid = get_process_by_name("YuanShen.exe").unwrap_or(0);
    if pid > 0 {
        if let Ok(killed) = kill_process(pid) {
            if killed {
                get_game(true).await;
            }
            return killed;
        }
    }
    false
}

#[tauri::command]
fn get_regsk() -> String {
    // 读取注册表
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = "Software\\miHoYo\\原神";
    let value = "MIHOYOSDK_ADL_PROD_CN_h3123967166";
    let sk = hkcu.open_subkey(key);
    if let Ok(sk) = sk {
        let val = sk.get_raw_value(value);
        if let Ok(val) = val {
            return String::from_utf8(val.bytes).unwrap_or(String::default());
        }
    }
    return "".to_string();
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct UIDCache {
    uid: String,
    usk: String,
    usd: String,
}

#[tauri::command]
fn get_uid() -> UIDCache {
    // 读取注册表
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = "Software\\miHoYo\\原神";
    let sk = hkcu.open_subkey_with_flags(key, KEY_READ);
    if let Ok(sk) = sk {
        let tpl = regex::Regex::new(r"^USD_\d\d+").unwrap();
        let values = sk
            .enum_values()
            .filter(|x| {
                if let Ok(x) = x {
                    x.0.starts_with("USD_") && !x.0.starts_with("USD_0")
                } else {
                    false
                }
            })
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        if values.len() > 1 {
            return UIDCache::default();
        }
        for (name, value) in values {
            let uid: Option<regex::Match> = tpl.find(&name);

            if let Some(uid) = uid {
                let usd = String::from_utf8(value.bytes).unwrap_or(String::default());
                let cache = UIDCache {
                    uid: uid.as_str()[4..].to_string(),
                    usk: name,
                    usd,
                };
                return cache; //serde_json::to_string(&cache).unwrap();
            }
        }
    }
    return UIDCache::default();
}

#[tauri::command]
fn set_usd(usk: String, usd: String) {
    // 读取注册表
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = "Software\\miHoYo\\原神";
    let sk = hkcu.open_subkey_with_flags(key, KEY_SET_VALUE | KEY_READ);
    if let Ok(sk) = sk {
        let val = RegValue {
            vtype: REG_BINARY,
            bytes: usd.as_bytes().to_vec(), //str.bytes().chain(std::iter::once(0)).collect(),
        };
        let _ = sk.set_raw_value(usk, &val);
    }
}

#[tauri::command]
fn set_regsk(str: String, uid: String) {
    // 写入注册表
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = "Software\\miHoYo\\原神";
    let value = "MIHOYOSDK_ADL_PROD_CN_h3123967166";
    let sk = hkcu.open_subkey_with_flags(key, KEY_SET_VALUE | KEY_READ);
    if let Ok(sk) = sk {
        // 删除所有UID缓存
        let val = sk
            .enum_values()
            .map(|x| x.unwrap().0)
            .filter(|x| x.starts_with("USD_"));
        for name in val {
            let tuid = regex::Regex::new(r"USD_\d\d+").unwrap().find(&name);
            if let Some(tuid) = tuid {
                if tuid.as_str()[4..].to_string() == uid {
                    continue;
                }
                let _ = sk.delete_value(name.clone());
                println!("Delete UID KEY: {}", name)
                // return uid.as_str()[4..].to_string();
            }
        }

        let val = RegValue {
            vtype: REG_BINARY,
            bytes: str.as_bytes().to_vec(), //str.bytes().chain(std::iter::once(0)).collect(),
        };
        let _ = sk.set_raw_value(value, &val);
    }
}

// 等待游戏启动状态变化再返回
#[tauri::command]
async fn get_game(is_run: bool) -> bool {
    let mut elapsed = Duration::from_secs(0);
    let timeout = Duration::from_secs(30);
    let interval = Duration::from_millis(500); // 500ms

    while elapsed <= timeout {
        let now_is_run = get_process_by_name("YuanShen.exe").unwrap_or(0) > 0;
        if now_is_run != is_run {
            return now_is_run;
        }
        tokio::time::sleep(interval).await;
        elapsed += interval;
    }
    is_run
}

#[tauri::command]
fn is_ingame() -> bool {
    if let Some(hwnd) = get_window_by_process_name(GAME_PROCESS) {
        let ctl = GameControl::new(hwnd, false);
        ctl.isForeground()
    } else {
        false
    }
}
#[tauri::command]
fn set_hotkey<R: Runtime>(app: AppHandle<R>, key: String) -> bool {
    // static mut HOOKID: isize = 0;
    // if unsafe { HOOKID } != 0 {
    //     del_keyboard_hook(unsafe { HOOKID });
    // }
    // set_keyboard_hook(keyboard_hook);
    true
}

/// 自动验证码
fn auto_slide(hwnd: isize) {
    fn ease_out_quart(x: f32) -> f32 {
        1. - (1. - x).powi(4)
    }

    fn get_tracks(distance: f32, seconds: f32) -> Vec<i32> {
        let mut tracks = vec![0];
        let mut last = 0;
        let mut t = 0.0;
        while t < seconds {
            let current = (ease_out_quart(t / seconds) * distance) as i32;
            tracks.push(current - last);
            last = current;
            t += 0.1;
        }
        tracks
    }

    fn ease_move(x: i32, t: f32) {
        let tx = x.abs() as f32;
        let tracks: Vec<i32> = get_tracks(tx, t);
        if x > 0 {
            for track in tracks {
                mouse_move(track, 0);
                sleep(20);
            }
        } else {
            for track in tracks {
                mouse_move(-track, 0);
                sleep(20);
            }
        }
    }

    let tx = (cap_slide(hwnd) as f32 * 1.05) as i32;
    if tx > 0 {
        let ctl = GameControl::new(hwnd, false);
        ctl.SavePos();
        ctl.MouseMove(719, 512);
        mouse_down();
        loop {
            let pos = ctl.MouseGetPos();
            let nx = pos.0 - 719 - 40;
            sleep(20);
            if nx.abs_diff(tx) < 5 {
                break;
            }
            if tx - nx > 500 {
                break;
            }
            ease_move((tx - nx).min(100), 1.);
        }
        mouse_up();
        sleep(20);
        ctl.RestorePos();
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("game")
        .invoke_handler(tauri::generate_handler![
            get_regsk,
            get_uid,
            set_usd,
            set_regsk,
            get_game,
            kill_game,
            auto_join,
            auto_open,
            auto_login,
            auto_setup,
            is_ingame,
            set_hotkey,
            launch_game
        ])
        .build()
}

#[cfg(test)]
mod tests {
    use super::{get_process_by_name, get_regsk, get_uid};

    #[test]
    fn test_get_procees() {
        let rst = get_process_by_name("YuanShen.exe").unwrap();
        println!("rst: {}", rst);
    }

    #[test]
    fn test_get_regsk() {
        let rst = get_regsk();
        println!("rst: {}", rst);
    }

    #[test]
    fn test_get_uid() {
        let rst = get_uid();
        println!("rst: {:?}", rst);
    }
    // #[test]
    // fn test_launch_game() {
    //     let rst = launch_game(
    //         "D:\\usr\\Games\\Genshin Impact\\Genshin Impact Game\\YuanShen.exe",
    //         "--",
    //         false,
    //         true,
    //         "".to_string(),
    //         "".to_string(),
    //     );
    //     println!("rst: {}", rst);
    // }
}
