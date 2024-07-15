use std::{ffi::c_void, time::Duration};

use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};
use winreg::{enums::*, RegKey, RegValue};

use self::util::*;
use game::GameControl;
mod game;
mod util;

#[derive(Serialize, Deserialize, Clone)]
struct LoginPayload {
    success: bool,
}

#[tauri::command]
pub fn launch_game<R: Runtime>(
    app: AppHandle<R>,
    path: &str,
    cmds: &str,
    unlock: bool,
    autologin: bool,
    login: String,
    pwd: String,
) -> bool {
    let pid = get_procees_by_name("YuanShen.exe").unwrap_or(0);
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
    let pi = shell_execute(path, Some(cmds), None);
    if let Err(err) = pi {
        println!("Failed to launch game: {:?}", err);
        return false;
    }
    let pi = pi.unwrap();
    println!("Process ID: {}", pi.hProcess);

    std::thread::spawn(move || {
        let login = login.clone();
        let pwd = pwd.clone();

        // 等待游戏窗口出现
        println!("等待游戏窗口出现");
        loop {
            if let Ok(npid) = get_procees_by_name("YuanShen.exe") {
                if let Some(hwnd) = get_window_by_process(npid) {
                    if pid > 0 {
                        move_window(hwnd, x, y);
                        if let Some(rect) = get_window_rect(hwnd) {
                            if rect.left == x && rect.top == y {
                                println!(
                                    "window_rect hwnd = {}, pos = {},{}",
                                    hwnd, rect.left, rect.top
                                );
                                break;
                            }
                        }
                    } else {
                        if let Some(rect) = get_window_rect(hwnd) {
                            if rect.right - rect.left > 500 {
                                break;
                            }
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        if autologin {
            if let Ok(npid) = get_procees_by_name("YuanShen.exe") {
                if let Some(hwnd) = get_window_by_process(npid) {
                    let ctl = GameControl::new(hwnd);
                    // 适龄提示
                    println!("自动登录");
                    if !ctl.WaitColor(1510, 70, "1.9.D.|0.4.6.", 30.) {
                        println!("登录超时");
                        let _ = app.emit("game_login", LoginPayload { success: false });
                        return;
                    }
                    println!("登录开始");
                    ctl.focus();
                    if !ctl.CheckColor(819, 838, "FFFFFF") {
                        println!("需输入密码");
                        ctl.WaitColor(626, 274, "FFFFFF", 5.); // 登陆框
                        ctl.Click(971, 348);
                        ctl.Sleep(100);
                        ctl.SendText(login.as_str());
                        ctl.Sleep(300);
                        ctl.Click(994, 420);
                        ctl.Sleep(100);
                        ctl.SendText(pwd.as_str());
                        ctl.Sleep(300);
                        if !ctl.CheckColor(580, 505, "DEBC60") {
                            ctl.Click(581, 514);
                            ctl.Sleep(100);
                        }
                        ctl.Click(973, 580);
                    }
                    // 点击进入
                    if ctl.WaitColor(762, 829, "FFFFFF", 10.) {
                        println!("登录成功");
                        ctl.Click(819, 838);
                        // 等待加载完毕
                        println!("等待加载");
                        ctl.WaitColor(51, 85, "4.6.A.", 20.);
                        if ctl.CheckColor(77, 49, "FFFFFF") // 主界面
                            && ctl.CheckColor(385, 58, "FFFFFF")
                        // F2为白色
                        {
                            println!("成功加载");
                            ctl.Click(385, 58); // 点击F2
                            if ctl.WaitColor(469, 840, "ECE5D8", 2.) {
                                println!("设置权限");
                                ctl.Click(469, 840); // 世界权限
                                ctl.Sleep(100);
                                ctl.Click(485, 735); // 直接加入
                                                     // ctl.Click(473, 785); // 确认后可加入
                                ctl.Sleep(100);
                                ctl.Click(469, 840); // 世界权限
                                println!("登录结束");
                                let _ = app.emit("game_login", LoginPayload { success: true });

                                return;
                            }
                        } else {
                            println!("未加载成功");
                        }
                    }
                    let _ = app.emit("game_login", LoginPayload { success: false });
                }
            }
        }
    });

    if unlock {
        let h_unity_player = get_module_by_name(pi.hProcess, "UnityPlayer.dll");
        if let Err(err) = h_unity_player {
            println!("Failed to get UnityPlayer.dll: {:?}", err);
            return false;
        }
        let h_unity_player = h_unity_player.unwrap();

        let pfps = get_memory_by_pattern(
            pi.hProcess,
            h_unity_player.modBaseAddr as *const c_void,
            h_unity_player.modBaseSize as usize,
            "7F 0E E8 ?? ?? ?? ?? 66 0F 6E C8",
            h_unity_player.modBaseAddr as usize,
        );

        if let Err(err) = pfps {
            println!("Failed to get FPS Offset: {:?}", err);
            return false;
        }
        let pfps = pfps.unwrap();
        println!("FPS Offset: {:?}", pfps);

        write_memory_until_exit(pi.hProcess, pfps, 140);
    } else {
        return true;
    }

    false
}

#[tauri::command]
async fn kill_game() -> bool {
    let pid = get_procees_by_name("YuanShen.exe").unwrap_or(0);
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
        let now_is_run = get_procees_by_name("YuanShen.exe").unwrap_or(0) > 0;
        if now_is_run != is_run {
            return now_is_run;
        }
        tokio::time::sleep(interval).await;
        elapsed += interval;
    }
    is_run
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
            launch_game
        ])
        .build()
}

#[cfg(test)]
mod tests {
    use super::{get_procees_by_name, get_regsk, get_uid};

    #[test]
    fn test_get_procees() {
        let rst = get_procees_by_name("YuanShen.exe").unwrap();
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
