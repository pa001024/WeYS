import { invoke } from "@tauri-apps/api/core"

export const MATERIALS = ["None", "Blur", "Acrylic", "Mica", "Mica_Dark", "Mica_Tabbed", "Mica_Tabbed_Dark"] as const
export async function applyMaterial(material: (typeof MATERIALS)[number]) {
    return (await invoke("apply_material", { material })) as string
}

export async function getOSVersion() {
    return (await invoke("get_os_version")) as string
}

/**
 * 获取本地登录的QQ号
 * @param port The port of the local QQ
 * @example 4301 4303 4305 ...
 * @returns JSON
 */
export async function getLocalQQ(port: number) {
    return JSON.parse((await invoke("get_local_qq", { port })) as string) as {
        uin: number
        face_index: number
        gender: number
        nickname: string
        client_type: number
        uin_flag: number
        account: number
    }[]
}
