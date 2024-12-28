import { invoke } from "@tauri-apps/api/core"

export async function getRegsk() {
    return (await invoke("plugin:game|get_regsk")) as string
}
export async function setRegsk(str: string, uid: string = "") {
    return (await invoke("plugin:game|set_regsk", { str, uid })) as void
}
export async function getGame(isRun: boolean) {
    return (await invoke("plugin:game|get_game", { isRun })) as boolean
}
export async function killGame() {
    return (await invoke("plugin:game|kill_game")) as boolean
}
export async function launchGame(id: string, path: string, cmds: string, unlock: boolean) {
    return (await invoke("plugin:game|launch_game", { id, path, cmds, unlock })) as boolean
}
export async function getUid() {
    return (await invoke("plugin:game|get_uid")) as { uid: string; usk: string; usd: string }
}
export async function setUsd(usk: string, usd: string) {
    return (await invoke("plugin:game|set_usd", { usk, usd })) as void
}
export async function autoJoin(uid: string) {
    return (await invoke("plugin:game|auto_join", { uid })) as boolean
}
// 1:自动 2:请求 3:关门
export async function autoOpen(state = 1, post = false, cloud = false) {
    return (await invoke("plugin:game|auto_open", { state, post, cloud })) as boolean
}
export async function autoLogin(id: string, login = "", pwd = "", post = false, cloud = false) {
    return (await invoke("plugin:game|auto_login", { id, login, pwd, post, cloud })) as boolean
}
export async function autoSetup(id: string, autosend = false, post = false, cloud = false) {
    return (await invoke("plugin:game|auto_setup", { id, autosend, post, cloud })) as boolean
}
export async function isIngame() {
    return (await invoke("plugin:game|is_ingame")) as boolean
}
export async function setHotkey(key: string) {
    return (await invoke("plugin:game|set_hotkey", { key })) as boolean
}
