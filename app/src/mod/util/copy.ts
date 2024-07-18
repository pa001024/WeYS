import { env } from "../../env"
import * as clipboard from "@tauri-apps/plugin-clipboard-manager"
import { htmlToText } from "./html"

export const copyText = async (text: string) => {
    if (env.isApp) {
        await clipboard.writeText(text)
        return
    }
    if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(text)
        return
    }
    const input = document.createElement("textarea")
    input.value = text
    input.style.position = "fixed"
    input.style.opacity = "0"
    document.body.appendChild(input)
    input.focus()
    input.select()
    const result = document.execCommand("copy")
    input.remove()
    if (!result) throw new Error("复制失败")
}

export const copyContent = async (text: string) => {
    await copyText(htmlToText(text))
}

export async function pasteText() {
    if (env.isApp) {
        return await clipboard.readText()
    }
    return await navigator.clipboard.readText()
}
