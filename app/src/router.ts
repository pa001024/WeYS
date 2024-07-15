import { RouteRecordRaw, createWebHashHistory, createRouter } from "vue-router"

import Home from "./views/Home.vue"
import Setting from "./views/Setting.vue"
import Game from "./views/Game.vue"
import User from "./views/User.vue"
import Chat from "./views/Chat.vue"
import ChatRoom from "./views/ChatRoom.vue"
import Download from "./views/Download.vue"
import { env } from "./env"

let setMinSize = async (_w: number, _h: number) => {}

;(async () => {
    if (!env.isApp) return
    const { LogicalSize, getCurrent } = await import("@tauri-apps/api/window")
    setMinSize = async (w: number, h: number) => {
        const win = getCurrent()
        win.setMinSize(new LogicalSize(w, h))
        const size = await win.innerSize()
        const factor = await win.scaleFactor()
        const logicalSize = size.toLogical(factor)
        win.setSize(new LogicalSize(Math.max(w, logicalSize.width), Math.max(h, logicalSize.height)))
    }
})()

const routes: readonly RouteRecordRaw[] = [
    { name: "home", path: "/", component: Home, beforeEnter: () => setMinSize(367, 430) },
    { name: "game", path: "/game", component: Game, beforeEnter: () => setMinSize(572, 500) },
    { name: "download", path: "/download", component: Download, beforeEnter: () => setMinSize(572, 500) },
    {
        name: "chat",
        path: "/chat",
        component: Chat,
        beforeEnter: () => setMinSize(367, 430),
        meta: { keepAlive: true },
        children: [
            { name: "room", path: ":room", component: ChatRoom }, //
        ],
    },
    { name: "user", path: "/user", component: User, beforeEnter: () => setMinSize(367, 430) },
    { name: "setting", path: "/setting", component: Setting, beforeEnter: () => setMinSize(540, 430) },
    { name: "voice", path: "/voice", component: import("./views/Voice.vue"), beforeEnter: () => setMinSize(540, 430) },
].filter((r) => env.isApp || r.name !== "game")

export const router = createRouter({
    history: createWebHashHistory(),
    routes,
})
