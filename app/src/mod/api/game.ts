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
export async function launchGame(path: string, cmds: string, unlock: boolean, autologin = false, login = "", pwd = "") {
    return (await invoke("plugin:game|launch_game", { path, cmds, unlock, autologin, login, pwd })) as boolean
}
export async function getUid() {
    return (await invoke("plugin:game|get_uid")) as { uid: string; usk: string; usd: string }
}
export async function setUsd(usk: string, usd: string) {
    return (await invoke("plugin:game|set_usd", { usk, usd })) as void
}
