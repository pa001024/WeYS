import { useLocalStorage } from "@vueuse/core"
import { defineStore } from "pinia"

export const useSettingStore = defineStore("setting", {
    state: () => {
        return {
            theme: useLocalStorage("setting_theme", "light"),
            uiScale: useLocalStorage("setting_ui_scale", 1),
            autoCount: useLocalStorage("setting_auto_count", false),
            minCountInterval: useLocalStorage("setting_min_count_interval", 50),
            windowTrasnparent: useLocalStorage("setting_window_trasnparent", true),
        }
    },
    getters: {
    },
    actions: {
        setTheme(theme: string) {
            this.theme = theme
        },
    },
})
