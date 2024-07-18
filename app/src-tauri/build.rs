use tauri_build::{Attributes, InlinedPlugin, WindowsAttributes};

fn main() {
    let attr = Attributes::new()
        .plugin(
            "game",
            InlinedPlugin::new().commands(&[
                "kill_game",
                "get_regsk",
                "get_uid",
                "set_usd",
                "set_regsk",
                "get_game",
                "auto_join",
                "auto_open",
                "launch_game",
            ]),
        )
        .windows_attributes(
            // 管理员权限运行
            WindowsAttributes::new().app_manifest(include_str!("../../client/misc/app.manifest")),
        );
    tauri_build::try_build(attr).expect("failed to run build script");
}
