<script lang="ts" setup>
import { computed, onBeforeMount, ref } from "vue"
import { useRoute, useRouter } from "vue-router"
import { t } from "i18next"
import { htmlToText } from "../mod/util/html"
import { createRoomMutation, deleteRoomMutation } from "../mod/api/mutation"
import { useUserStore } from "../mod/state/user"

const user = useUserStore()
const router = useRouter()
const route = useRoute()
const search = ref("")
const newroom = ref("")
const newroomType = ref("normal")
const newroomMaxUser = ref("0")
const actived = computed(() => route.params.room)
const variables = computed(() => ({ name_like: search.value.trim() || null }))

const Query = /* GraphQL */ `
    query ($name_like: String, $limit: Int, $offset: Int) {
        rooms(name_like: $name_like, limit: $limit, offset: $offset) {
            id
            name
            type
            updateAt
            maxUsers
            msgCount
            owner {
                id
                name
                qq
            }
            lastMsg {
                id
                content
                createdAt
                user {
                    id
                    name
                    qq
                }
            }
        }
    }
`

interface Room {
    id: string
    name: string
    type: string
    updateAt: string
    maxUsers: string
    owner: string
    msgs: any[]
}

async function createRoom() {
    const result = await createRoomMutation({
        name: newroom.value.trim(),
        type: newroomType.value,
        maxUsers: +newroomMaxUser.value || void 0,
    })
    if (result) {
        newroom.value = ""
        await reloadRooms()
        router.push({ name: "room", params: { room: result.id } })
    }
}

function toLocaleTimeString(timestamp: number) {
    const date = new Date(timestamp)
    if (date.toLocaleDateString() !== new Date().toLocaleDateString()) {
        return date.toLocaleDateString()
    }
    return date.toLocaleTimeString().slice(0, -3)
}

const handle = ref<null | (() => Promise<void>)>(null)
async function reloadRooms() {
    await handle.value!()
}

async function enterRoom(room: Room) {
    router.push({ name: "room", params: { room: room.id } })
}
async function deleteRoom(room: Room) {
    if ((await confirm(t("chat.deleteRoomConfirm"))) && (await deleteRoomMutation({ id: room.id }))) {
        await reloadRooms()
    } else {
        alert(t("chat.deleteRoomFailed"))
    }
}

const toggleMenu = ref(true)

onBeforeMount(() => {
    if (!user.id) {
        router.push({ name: "user", query: { tip: t("chat.needLogin") } })
    }
})
</script>

<template>
    <div class="w-full h-full flex overflow-hidden">
        <transition name="slide-right">
            <div
                class="flex flex-col flex-none overflow-hidden w-full sm:w-60 sm:shadow-md"
                :class="{ 'max-sm:hidden': actived }"
                v-show="toggleMenu"
            >
                <div class="flex p-2 gap-2 bg-base-100/30">
                    <!-- 搜索 -->
                    <label class="text-base-content/70 flex-1 input input-ghost input-sm flex items-center gap-2 bg-base-200">
                        <Icon icon="la:search-solid" />
                        <input type="text" class="flex-1 w-8" :placeholder="$t('chat.search')" v-model="search" />
                    </label>
                    <Tooltip :tooltip="$t('chat.addRoom')" side="bottom">
                        <Popover class="btn btn-square btn-sm" :title="$t('chat.addRoom')" @confirm="createRoom">
                            <Icon icon="la:plus-solid" />
                            <template #content>
                                <div class="form-control p-2">
                                    <label class="label">
                                        <span class="label-text">{{ $t("chat.roomName") }}</span>
                                        <input v-model="newroom" type="text" class="input input-bordered input-sm" />
                                    </label>

                                    <label class="label">
                                        <span class="label-text">{{ $t("chat.roomType") }}</span>
                                        <Select v-model="newroomType" :placeholder="$t('chat.roomType')">
                                            <SelectItem value="normal">{{ $t("chat.normal") }}</SelectItem>
                                            <SelectItem value="legend">{{ $t("chat.legend") }}</SelectItem>
                                        </Select>
                                    </label>
                                    <label class="label">
                                        <span class="label-text">{{ $t("chat.roomMaxUser") }}</span>
                                        <Select v-model="newroomMaxUser" :placeholder="$t('chat.roomMaxUser')">
                                            <SelectItem value="0">{{ $t("chat.unlimited") }}</SelectItem>
                                            <SelectItem v-for="i in 9" :value="String(i)">{{ i }}</SelectItem>
                                        </Select>
                                    </label>
                                </div>
                            </template>
                        </Popover>
                    </Tooltip>
                    <Tooltip :tooltip="$t('chat.refresh')" side="bottom">
                        <div class="btn btn-square btn-sm" @click="reloadRooms">
                            <Icon icon="la:sync-solid" />
                        </div>
                    </Tooltip>
                </div>
                <!-- 列表 -->
                <GQAutoPage
                    @load="handle = $event"
                    class="flex-1 overflow-hidden"
                    :size="10"
                    :query="Query"
                    :variables="variables"
                    dataKey="rooms"
                    v-slot="{ data }"
                >
                    <ContextMenu v-if="data" v-for="r in data.rooms" :key="r.id">
                        <div
                            @click="enterRoom(r)"
                            class="h-16 bg-base-100/50 p-4 flex flex-col justify-center group"
                            :class="{ 'active bg-primary': actived === r.id }"
                        >
                            <!-- 房间名 -->
                            <div class="flex justify-between items-center group-[.active]:text-base-100 gap-2">
                                <div
                                    v-if="r.type && r.type !== 'normal'"
                                    class="whitespace-nowrap text-xs rounded-lg bg-primary text-base-100 px-2"
                                >
                                    {{ $t(`chat.${r.type}`) }}
                                </div>
                                <div class="whitespace-nowrap flex-1 overflow-hidden text-ellipsis text-sm">{{ r.name }}</div>
                                <div class="text-xs text-base-content/70 group-[.active]:text-base-100">
                                    {{ toLocaleTimeString(r.lastMsg?.createdAt || r.updateAt) }}
                                </div>
                            </div>
                            <!-- 最新消息 -->
                            <div class="flex justify-between group-[.active]:text-base-100 text-base-content/50 space-x-4">
                                <div v-if="r.lastMsg" class="whitespace-nowrap max-w-36 text-xs flex-1 overflow-hidden text-ellipsis">
                                    {{ r.lastMsg.user.name }}: {{ htmlToText(r.lastMsg.content) }}
                                </div>
                                <div
                                    v-if="r.msgCount - (user.roomReadedCount[r.id] || 0) > 0"
                                    class="whitespace-nowrap text-xs font-bold text-base-100 rounded-lg bg-primary px-2 group-[.active]:hidden"
                                >
                                    {{ r.msgCount - (user.roomReadedCount[r.id] || 0) }}
                                </div>
                            </div>
                        </div>

                        <template #menu>
                            <ContextMenuItem
                                :disabled="r.owner.id !== user.id"
                                @click="deleteRoom(r)"
                                class="group text-sm p-2 leading-none text-base-content rounded flex items-center relative select-none outline-none data-[disabled]:text-base-content/60 data-[disabled]:pointer-events-none data-[highlighted]:bg-primary data-[highlighted]:text-base-100"
                            >
                                <Icon class="size-4 mr-2" icon="la:trash-alt" />
                                {{ $t("room.remove") }}
                            </ContextMenuItem>
                        </template>
                    </ContextMenu>
                </GQAutoPage>
            </div>
        </transition>
        <!-- 展开按钮 -->
        <div class="h-full flex flex-none relative items-center max-sm:hidden group">
            <div class="absolute z-10">
                <button
                    class="btn btn-ghost m-1 w-6 p-1 h-32 btn-sm opacity-0 group-hover:opacity-100 transition-opacity duration-500"
                    @click="toggleMenu = !toggleMenu"
                >
                    <span
                        class="flex-none items-center justify-center text-lg text-base-content/50 swap swap-flip"
                        :class="{ 'swap-active': toggleMenu }"
                    >
                        <Icon icon="tabler:arrow-bar-to-left" class="swap-on" />
                        <Icon icon="tabler:arrow-bar-to-right" class="swap-off" />
                    </span>
                </button>
            </div>
        </div>
        <div class="grow" :class="[actived ? 'max-sm:block' : 'max-sm:hidden']">
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
    </div>
</template>
