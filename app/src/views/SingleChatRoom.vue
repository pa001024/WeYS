<script lang="ts" setup>
import { computed, onBeforeMount, watch } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useUserStore } from "../mod/state/user"
import { t } from "i18next"
import { useSettingStore } from "../mod/state/setting"

const user = useUserStore()
const setting = useSettingStore()
const router = useRouter()
const route = useRoute()
const actived = computed(() => route.params.room)

watch(() => setting.winMaterial, setting.resetWinMaterial)
onBeforeMount(() => {
    if (!user.id) {
        router.push({ name: "user", query: { tip: t("chat.needLogin") } })
    }
})
</script>

<template>
    <div class="w-full h-full flex overflow-hidden">
        <RouterView v-slot="{ Component, route }">
            <transition name="slide-right">
                <Suspense>
                    <component :is="Component" :key="route.path" :room="actived" />
                    <template #fallback>
                        <div class="w-full h-full flex justify-center items-center">
                            <span class="loading loading-spinner loading-md"></span>
                        </div>
                    </template>
                </Suspense>
            </transition>
        </RouterView>
    </div>
</template>
