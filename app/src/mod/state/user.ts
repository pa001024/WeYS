import { useLocalStorage } from "@vueuse/core"
import { defineStore } from "pinia"
import { useObservable } from "@vueuse/rxjs"
import { liveQuery } from "dexie"
import { db } from "../db"

function getPayload(token: string) {
    if (!token) return
    function safeParse<T>(s: string) {
        try {
            return JSON.parse(atob(s.split(".")[1])) as T
        } catch (e) {
            return undefined
        }
    }
    return safeParse<{
        id: string
        name: string
        email: string
        qq: string
    }>(token)
}

export const useUserStore = defineStore("user", {
    state: () => {
        return {
            token: useLocalStorage("setting_token", ""),
            gameDay: useLocalStorage("gameDay", new Date(Date.now() - 4 * 36e5).toLocaleDateString("zh-CN")),
            // 每日怪数量 精英怪
            daily3000: useLocalStorage("daily_3000", 0),
            daily1200: useLocalStorage("daily_1200", 0),
            daily600: useLocalStorage("daily_600", 0),
            daily200: useLocalStorage("daily_200", 0),
            dailyNormal: useLocalStorage("daily_normal", 0),
            roomReadedCount: useLocalStorage("room_readed_count", {} as { [roomId: string]: number }),
            accounts: useObservable(liveQuery(() => db.accounts.toArray())),
        }
    },
    getters: {
        id(state) {
            return getPayload(state.token)?.id
        },
        email(state) {
            return getPayload(state.token)?.email
        },
        name(state) {
            return getPayload(state.token)?.name
        },
        qq(state) {
            return getPayload(state.token)?.qq
        },
        isGuest(state) {
            return getPayload(state.token)?.email.endsWith("guest")
        },
        today() {
            return new Date(Date.now() - 4 * 36e5).toLocaleDateString("zh-CN")
        },
        // 每日收益
        dailyIncome(state) {
            return state.daily3000 * 3000 + state.daily1200 * 1200 + state.daily600 * 600 + state.daily200 * 200
        },
        // 精英怪
        dailyElite(state) {
            return state.daily3000 * 3 + state.daily1200 * 2 + state.daily600 + state.daily200
        },
    },
    actions: {
        writeData() {
            // 写入数据库
            console.log("write data to database")
        },
        updateData(key: keyof typeof useUserStore, value: any) {
            this[key] = value
            // 检查是否为新的一天
            if (this.today !== this.gameDay) {
                this.writeData()
                this.resetDailyData()
            }
        },
        resetDailyData() {
            this.gameDay = this.today
            this.daily3000 = 0
            this.daily1200 = 0
            this.daily600 = 0
            this.daily200 = 0
            this.dailyNormal = 0
        },
        logout() {
            this.token = ""
            localStorage.removeItem("setting_token")
        },
        setRoomReadedCount(roomId: string, count: number) {
            this.roomReadedCount[roomId] = count
        },
        getRoomReadedCount(roomId: string) {
            return this.roomReadedCount[roomId] || 0
        },
    },
})
