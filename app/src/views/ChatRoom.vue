<script lang="ts" setup>
import { computed, onMounted, onUnmounted, reactive, ref, watch, watchEffect } from "vue"
import { useRoute } from "vue-router"
import { useTimestamp } from "@vueuse/core"
import { useScroll } from "@vueuse/core"
import { useSound } from "@vueuse/sound"
import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut"
import { env } from "../env"
import { gql, useQuery, useSubscription } from "@urql/vue"
import { isImage, sanitizeHTML } from "../mod/util/html"
import { copyContent, copyText, pasteText } from "../mod/util/copy"
import { addTaskMutation, editMessageMutation, endTaskMutation, joinTaskMutation, sendMessageMutation } from "../mod/api/mutation"
import { useUIStore } from "../mod/state/ui"
import { useUserStore } from "../mod/state/user"
import { useRTC } from "../mod/api/rtc"

const route = useRoute()
const roomId = computed(() => route.params.room as string)
const user = useUserStore()
const ui = useUIStore()
const newMsgTip = ref(true)
const newMsgJoin = ref(true)
const keyboardShortcut = ref(false)
const keyboardShortcutSetting = reactive({
    open: false,
    ctrl: true,
    shift: true,
    alt: false,
    key: "R",
})
let lastTask: Task | null = null

// const

watch(keyboardShortcut, async (enable) => {
    if (enable) {
        let key = keyboardShortcutSetting.key
        if (keyboardShortcutSetting.alt) key = "Alt+" + key
        if (keyboardShortcutSetting.shift) key = "Shift+" + key
        if (keyboardShortcutSetting.ctrl) key = "CmdOrControl+" + key
        await register(key, (e) => {
            if (e.state === "Pressed") {
                if (lastTask) {
                    autoJoinGame(lastTask)
                }
            }
        })
    } else {
        await unregisterAll()
    }
})

onUnmounted(async () => {
    await unregisterAll()
})

const el = ref<HTMLElement | null>(null)

const { arrivedState } = useScroll(el, { offset: { left: 0, top: 20, right: 0, bottom: 200 } })

const { data } = useQuery<{
    room: {
        id: string
        name: string
        maxUsers: number
        msgCount: number
    }
}>({
    query: gql`
        query ($roomId: String!) {
            room(id: $roomId) {
                id
                name
                maxUsers
                msgCount
            }
        }
    `,
    variables: { roomId },
    requestPolicy: "cache-first",
})
const isJoined = computed(() => !!data.value?.room || false)
const msgCount = computed(() => data.value?.room?.msgCount || 0)
const title = computed(() => data.value?.room?.name || "")
const maxUsers = computed(() => data.value?.room?.maxUsers || 0)

watchEffect(() => {
    if (msgCount.value) user.setRoomReadedCount(roomId.value, msgCount.value)
    if (title.value) ui.schatTitle = maxUsers.value ? `${title.value} (${maxUsers.value})` : `${title.value}`
})

const { micOn, state } = useRTC(roomId)
onMounted(async () => {
    if (msgCount.value > 0) user.setRoomReadedCount(roomId.value, msgCount.value)
})

const variables = computed(() => ({ roomId: roomId.value }))

type Msg = {
    id: string
    edited: number
    content: string
    createdAt: string
    user: { id: string; name: string; qq: number }
}

type Task = {
    id: string
    name: string
    desc: string
    maxUser: number
    userList: string[]
    startTime: string
    endTime: string
    roomId: string
    userId: string
    createdAt: string
    updateAt: string
    user: { id: string; name: string; qq: number }
}

useSubscription<{ newMessage: Msg }>(
    {
        query: gql`
            subscription ($roomId: String!) {
                newMessage(roomId: $roomId) {
                    id
                    edited
                    content
                    createdAt
                    user {
                        id
                        name
                        qq
                    }
                }
            }
        `,
        variables: { roomId },
    },
    (_, data) => {
        if (data.newMessage) {
            addMessage(data.newMessage)
        }
        return data
    }
)

useSubscription<{ msgEdited: Msg }>({
    query: gql`
        subscription ($roomId: String!) {
            msgEdited(roomId: $roomId) {
                id
                edited
                content
                createdAt
                user {
                    id
                    name
                    qq
                }
            }
        }
    `,
    variables: { roomId },
})

const newTaskSound = useSound("/sfx/notice.mp3")

useSubscription<{ newTask: Task }>(
    {
        query: gql`
            subscription ($roomId: String!) {
                newTask(roomId: $roomId) {
                    id
                    name
                    desc
                    maxUser
                    maxAge
                    userList
                    startTime
                    endTime
                    createdAt
                    updateAt
                    user {
                        id
                        name
                        qq
                    }
                }
            }
        `,
        variables: { roomId },
    },
    (_, data) => {
        if (data.newTask) {
            lastTask = data.newTask
            // 播放音效
            if (newMsgTip.value) newTaskSound.play()
            // 自动复制
            if (newMsgJoin.value) {
                copyText(data.newTask.name)
            }
        }
        return data
    }
)

useSubscription<{ updateTask: Task }>({
    query: gql`
        subscription ($roomId: String!) {
            updateTask(roomId: $roomId) {
                id
                name
                desc
                maxUser
                userList
                startTime
                endTime
                createdAt
                updateAt
                user {
                    id
                    name
                    qq
                }
            }
        }
    `,
    variables: { roomId },
})

const time = useTimestamp({ interval: 1000, offset: 0 })
const nowSeconds = computed(() => ~~(time.value / 1000))

async function addMessage(msg: Msg) {
    console.debug("addMessage", msg)
    if (arrivedState.bottom) {
        await new Promise((resolve) => setTimeout(resolve, 50))
        el.value?.scrollTo({
            top: el.value.scrollHeight,
            left: 0,
            behavior: "smooth",
        })
    }
}

const input = ref<HTMLDivElement>(null as any)
const inputForm = ref<HTMLDivElement>(null as any)
const newMsgText = ref("")

async function sendMessage(e: Event) {
    if ((e as KeyboardEvent)?.shiftKey) {
        return
    }
    e.preventDefault()
    const html = input.value?.innerHTML
    if (!html) return
    const content = sanitizeHTML(html)
    if (!content) return
    input.value.innerHTML = ""
    input.value.focus()
    await sendMessageMutation({ content, roomId: roomId.value })
    el.value?.scrollTo({
        top: el.value.scrollHeight,
        left: 0,
        behavior: "smooth",
    })
}

function insertEmoji(text: string) {
    const el = input.value
    el.focus()
    const sel = window.getSelection()!
    const range = sel.getRangeAt(0)
    const node = document.createElement("div")
    node.innerText = text
    let frag = document.createDocumentFragment()
    while (node.firstChild) frag.appendChild(node.firstChild)
    range.deleteContents()
    range.insertNode(frag)
    range.collapse(false)
}

async function insertImage() {
    const fi = document.createElement("input")
    fi.type = "file"
    fi.click()
    fi.onchange = async (e) => {
        const file = (e.target as HTMLInputElement).files?.[0]
        if (!file) return
        const reader = new FileReader()
        reader.readAsDataURL(file)
        reader.onload = async (e) => {
            const data = e.target!.result as string
            const el = input.value
            el.focus()
            const sel = window.getSelection()!
            const range = sel.getRangeAt(0)
            const node = document.createElement("div")
            node.innerHTML = `<img src="${data}" />`
            let frag = document.createDocumentFragment()
            while (node.firstChild) frag.appendChild(node.firstChild)
            range.deleteContents()
            range.insertNode(frag)
            range.collapse(false)
        }
    }
}

async function addTask() {
    const uid = await pasteText()
    if (uid.match(/^\d{9}$/)) {
        await addTaskMutation({
            roomId: roomId.value,
            name: uid,
            maxUser: 3,
            maxAge: 15,
            desc: "软饭",
        })
    }
}

let timerAutoTask = ref(0)
function startAutoTask() {
    let lastUID = ""
    if (timerAutoTask.value) {
        clearInterval(timerAutoTask.value)
        timerAutoTask.value = 0
    } else {
        timerAutoTask.value = setInterval(async () => {
            const uid = await pasteText()
            if (uid.match(/^\d{9}$/) && lastUID !== uid) {
                lastUID = uid
                await addTaskMutation({
                    roomId: roomId.value,
                    name: uid,
                    maxUser: 3,
                    maxAge: 15,
                    desc: "软饭",
                })
            }
        }, 100) as unknown as number
    }
}

onUnmounted(() => {
    clearInterval(timerAutoTask.value)
})

async function endTask(task: Task) {
    await endTaskMutation({ taskId: task.id })
}

const editId = ref("")
const editInput = ref<HTMLDivElement[] | null>(null)
const retractCache = new Map<string, string>()
async function retractMessage(msg: Msg) {
    retractCache.set(msg.id, msg.content)
    await editMessageMutation({ content: "", msgId: msg.id })
    msg.content = ""
}
async function restoreMessage(msg: Msg) {
    const content = retractCache.get(msg.id)
    msg.content = content || msg.content
    startEdit(msg)
}
async function startEdit(msg: Msg) {
    editId.value = msg.id
    await new Promise((resolve) => setTimeout(resolve, 50))
    if (editInput.value?.[0]) {
        let el = editInput.value[0]
        el.focus()
        el.onblur = async () => {
            const newVal = sanitizeHTML(el.innerHTML || "")
            if (msg.content === newVal) return
            await editMessageMutation({ content: newVal, msgId: editId.value })
            msg.content = newVal
            msg.edited = 1
            editId.value = ""
            el.onblur = null
            el.onkeydown = null
        }
        el.onkeydown = (e: KeyboardEvent) => {
            if (!e.ctrlKey && !e.altKey && !e.shiftKey && e.key === "Enter") {
                e.preventDefault()
                el.blur()
            }
        }
    }
}

async function autoJoinGame(task: Task) {
    if (!task.userList.includes(user.id!)) await joinTaskMutation({ taskId: task.id })
    // if (env.isApp && game.running) {
    //     await autoJoin(task.name)
    // } else {
    await copyText(task.name)
    // }
}
</script>

<template>
    <div class="w-full h-full bg-base-200/50 flex">
        <!-- 聊天窗口 -->
        <div v-if="isJoined && state === 'loaded'" class="flex-1 flex flex-col overflow-hidden">
            <!-- 主内容区 -->
            <div class="flex-1 flex flex-col overflow-hidden relative">
                <div class="flex flex-col gap-2 absolute left-0 right-0 justify-center z-10 p-4">
                    <!-- 任务列表 -->
                    <GQQuery
                        :query="`query ($roomId: String!) {
    doingTasks(roomId: $roomId) {
        id,name,desc,maxUser,maxAge,userList,startTime,endTime,createdAt,updateAt,user { id, name, qq }
    }
}`"
                        :variables="variables"
                        v-slot="{ data }"
                    >
                        <transition-group name="slide-right">
                            <div
                                v-if="data"
                                v-for="item in data.doingTasks"
                                :key="item"
                                class="flex items-center justify-between gap-2 bg-base-100 shadow-md rounded-md px-4 p-2"
                            >
                                <div class="flex flex-col">
                                    <div class="flex items-center gap-2">
                                        <span v-if="item.desc" class="whitespace-nowrap text-xs rounded-xl bg-primary text-base-100 px-2">{{
                                            item.desc
                                        }}</span>
                                        <span class="select-all">{{ item.name }}</span>
                                    </div>
                                    <div class="text-sm">
                                        {{ item.user.name }}
                                    </div>
                                </div>
                                <div class="flex flex-col gap-2 min-w-24 items-center">
                                    <div class="flex gap-1">
                                        <UserItem v-for="u in item.userList" :key="u" :id="u"></UserItem>
                                    </div>
                                    <div class="text-sm">{{ item.userList.length }} / {{ item.maxUser }}</div>
                                </div>
                                <div class="flex-none flex gap-2">
                                    <div class="btn btn-sm btn-primary" @click="endTask(item)">
                                        {{ $t("task.end") }}
                                    </div>
                                    <div v-if="env.isApp" class="btn btn-sm btn-primary" @click="autoJoinGame(item)">
                                        {{ $t("task.join") }}
                                    </div>
                                    <div v-else class="btn btn-sm btn-primary" @click="autoJoinGame(item)">
                                        {{ $t("task.copy") }}
                                    </div>
                                </div>
                            </div>
                        </transition-group>
                    </GQQuery>
                </div>
                <GQAutoPage
                    v-if="msgCount"
                    @loadref="(r) => (el = r)"
                    direction="top"
                    class="flex-1 overflow-hidden"
                    innerClass="flex w-full h-full flex-col gap-2 p-4"
                    :limit="20"
                    :offset="msgCount"
                    :query="`query ($roomId: String!, $limit: Int, $offset: Int) {
    msgs(roomId: $roomId, limit: $limit, offset: $offset) {
        id, edited, content, createdAt, user { id, name, qq }
    }
}
`"
                    :variables="variables"
                    dataKey="msgs"
                    v-slot="{ data }"
                >
                    <!-- 消息列表 -->
                    <div v-if="data" class="group flex items-start gap-2" v-for="item in data.msgs" :key="item.id">
                        <div v-if="!item.content && editId !== item.id" class="text-xs text-base-content/60 m-auto">
                            {{ $t("chat.retractedAMessage", { name: user.id === item.user.id ? $t("chat.you") : item.user?.name }) }}
                            <span class="text-xs text-primary underline cursor-pointer" @click="restoreMessage(item)">{{
                                $t("chat.restore")
                            }}</span>
                        </div>
                        <div class="flex-1 flex items-start gap-2" :class="{ 'flex-row-reverse': user.id === item.user.id }" v-else>
                            <ContextMenu>
                                <QQAvatar class="mt-2 size-8" :qq="item.user.qq" :name="item.user?.name"></QQAvatar>

                                <template #menu>
                                    <ContextMenuItem
                                        class="group text-sm p-2 leading-none text-base-content rounded flex items-center relative select-none outline-none data-[disabled]:text-base-content/60 data-[disabled]:pointer-events-none data-[highlighted]:bg-primary data-[highlighted]:text-base-100"
                                    >
                                        <Icon class="size-4 mr-2" icon="la:eye-slash" />
                                        {{ $t("chat.block") }}
                                    </ContextMenuItem>
                                    <ContextMenuItem
                                        class="group text-sm p-2 leading-none text-base-content rounded flex items-center relative select-none outline-none data-[disabled]:text-base-content/60 data-[disabled]:pointer-events-none data-[highlighted]:bg-primary data-[highlighted]:text-base-100"
                                    >
                                        <Icon class="size-4 mr-2" icon="la:glasses" />
                                        {{ $t("chat.follow") }}
                                    </ContextMenuItem>
                                </template>
                            </ContextMenu>
                            <ContextMenu class="flex items-start flex-col" :class="{ 'items-end': user.id === item.user.id }">
                                <div class="text-base-content/60 text-sm min-h-5">{{ item.user.name }}</div>
                                <div
                                    v-if="editId === item.id"
                                    ref="editInput"
                                    contenteditable
                                    class="safe-html rounded-lg bg-base-100 select-text inline-flex flex-col text-sm max-w-80 overflow-hidden gap-2"
                                    :class="{ 'p-2': !isImage(item.content), 'bg-primary text-base-100': user.id === item.user.id }"
                                    v-html="sanitizeHTML(item.content)"
                                ></div>
                                <div
                                    v-else
                                    class="safe-html rounded-lg bg-base-100 select-text inline-flex flex-col text-sm max-w-80 overflow-hidden gap-2"
                                    :class="{ 'p-2': !isImage(item.content), 'bg-primary text-base-100': user.id === item.user.id }"
                                    v-html="sanitizeHTML(item.content)"
                                ></div>

                                <template #menu>
                                    <ContextMenuItem
                                        @click="copyContent(item.content)"
                                        class="group text-sm p-2 leading-none text-base-content rounded flex items-center relative select-none outline-none data-[disabled]:text-base-content/60 data-[disabled]:pointer-events-none data-[highlighted]:bg-primary data-[highlighted]:text-base-100"
                                    >
                                        <Icon class="size-4 mr-2" icon="la:copy-solid" />
                                        {{ $t("chat.copy") }}
                                    </ContextMenuItem>
                                    <ContextMenuItem
                                        v-if="user.id === item.user.id"
                                        @click="retractMessage(item)"
                                        class="group text-sm p-2 leading-none text-base-content rounded flex items-center relative select-none outline-none data-[disabled]:text-base-content/60 data-[disabled]:pointer-events-none data-[highlighted]:bg-primary data-[highlighted]:text-base-100"
                                    >
                                        <Icon class="size-4 mr-2" icon="la:reply-solid" />
                                        {{ $t("chat.revert") }}
                                    </ContextMenuItem>
                                    <ContextMenuItem
                                        v-if="user.id === item.user.id"
                                        @click="startEdit(item)"
                                        class="group text-sm p-2 leading-none text-base-content rounded flex items-center relative select-none outline-none data-[disabled]:text-base-content/60 data-[disabled]:pointer-events-none data-[highlighted]:bg-primary data-[highlighted]:text-base-100"
                                    >
                                        <Icon class="size-4 mr-2" icon="la:edit-solid" />
                                        {{ $t("chat.edit") }}
                                    </ContextMenuItem>
                                </template>
                            </ContextMenu>
                            <div class="text-xs text-base-content/60 self-end" v-if="item.edited">{{ $t("chat.edited") }}</div>
                            <div class="flex-1"></div>
                            <div class="hidden group-hover:block p-1 text-xs text-base-content/60">{{ item.createdAt }}</div>
                        </div>
                    </div>
                </GQAutoPage>
                <div v-else class="flex-1 flex flex-col items-center justify-center">
                    <div class="flex-1 flex flex-col items-center justify-center">
                        <div class="flex p-4 font-bold text-lg text-base-content/60">{{ $t("chat.newRoomBanner") }}</div>
                        <div class="flex btn btn-primary" @click="sendMessageMutation({ content: $t('chat.hello'), roomId })">
                            {{ $t("chat.sayHello") }}
                        </div>
                    </div>
                </div>
            </div>
            <!-- 在线用户 -->
            <GQQuery
                :query="`query ($roomId: String!) {
    rtcClients(roomId: $roomId) {
        id, end, user { id, name, qq }
    }
}`"
                :variables="variables"
                v-slot="{ data }"
            >
                <div v-if="data" class="flex items-center p-1 gap-1">
                    <div class="flex group bg-primary items-center rounded-full px-1">
                        <QQAvatar class="size-6 my-1" :name="user.name!" :qq="user.qq!" />
                        <div
                            class="flex text-sm text-base-100 max-w-0 group-hover:max-w-24 group-hover:mx-1 overflow-hidden transition-all duration-500 whitespace-nowrap"
                        >
                            {{ user.name }}
                        </div>
                    </div>
                    <div
                        v-for="item in data.rtcClients.filter((v:any) => v.user.id !== user.id)"
                        :key="item.id"
                        class="flex group bg-primary items-center rounded-full px-1"
                    >
                        <QQAvatar class="size-6 my-1" :name="item.user.name" :qq="item.user.qq" />
                        <div
                            class="flex text-sm text-base-100 max-w-0 group-hover:max-w-24 group-hover:mx-1 overflow-hidden transition-all duration-500 whitespace-nowrap"
                        >
                            {{ item.user.name }}
                        </div>
                    </div>
                </div>
            </GQQuery>
            <!-- 分割线 -->
            <div class="flex-none w-full relative">
                <div
                    class="w-full absolute -mt-[3px] h-[6px] cursor-ns-resize z-100"
                    v-h-resize-for="{ el: inputForm, min: 120, max: 400 }"
                ></div>
            </div>
            <!-- 输入 -->
            <form class="h-44 flex flex-col relative border-t-[1px] border-base-300/50" ref="inputForm" @submit="sendMessage">
                <!-- 工具栏 -->
                <div class="flex-none p-1 px-2 border-t-[1px] border-base-300/50 flex items-center gap-2">
                    <!-- 表情 -->
                    <Popover side="top">
                        <div class="btn btn-ghost btn-sm btn-square text-xl hover:text-primary">
                            <Icon icon="la:smile" />
                        </div>
                        <template #content>
                            <EmojiSelect @select="insertEmoji" />
                        </template>
                    </Popover>
                    <Tooltip side="top" :tooltip="$t('chat.insertImage')">
                        <div class="btn btn-ghost btn-sm btn-square text-xl hover:text-primary" @click="insertImage">
                            <Icon icon="la:image-solid" />
                        </div>
                    </Tooltip>
                    <Tooltip side="top" :tooltip="$t('chat.sendTask')">
                        <CheckAnimationButton
                            :class="{ 'text-primary': timerAutoTask }"
                            icon="la:plus-solid"
                            @click="addTask"
                            @contextmenu.prevent="startAutoTask"
                        />
                    </Tooltip>
                    <Tooltip side="top" :tooltip="$t('chat.sound')">
                        <div
                            class="btn btn-ghost btn-sm btn-square text-xl"
                            :class="{ 'text-primary': newMsgTip }"
                            @click="newMsgTip = !newMsgTip"
                        >
                            <Icon icon="la:volume-up-solid" />
                        </div>
                    </Tooltip>
                    <Tooltip side="top" :tooltip="$t('chat.autoJoin')">
                        <div
                            class="btn btn-ghost btn-sm btn-square text-xl"
                            :class="{ 'text-primary': newMsgJoin }"
                            @click="newMsgJoin = !newMsgJoin"
                        >
                            A
                        </div>
                    </Tooltip>
                    <Tooltip side="top" :tooltip="$t('chat.micOn')">
                        <div class="btn btn-ghost btn-sm btn-square text-xl" :class="{ 'text-primary': micOn }" @click="micOn = !micOn">
                            <Icon icon="la:microphone-solid" />
                        </div>
                    </Tooltip>

                    <Popover
                        v-model="keyboardShortcutSetting.open"
                        :title="$t('chat.keyboardShortcutSetting')"
                        noconfirm
                        @contextmenu.prevent="keyboardShortcutSetting.open = true"
                    >
                        <Tooltip v-if="env.isApp" side="top" :tooltip="$t('chat.keyboardShortcut')">
                            <div
                                class="btn btn-ghost btn-sm btn-square text-xl"
                                :class="{ 'text-primary': keyboardShortcut }"
                                @click.stop="keyboardShortcut = !keyboardShortcut"
                            >
                                <Icon icon="la:keyboard" />
                            </div>
                        </Tooltip>
                        <template #content>
                            <div class="form-control p-2">
                                <div class="label">
                                    <span class="label-text">Ctrl</span>
                                    <input v-model="keyboardShortcutSetting.ctrl" type="checkbox" class="toggle toggle-secondary" />
                                </div>
                                <div class="label">
                                    <span class="label-text">Shift</span>
                                    <input v-model="keyboardShortcutSetting.shift" type="checkbox" class="toggle toggle-secondary" />
                                </div>
                                <div class="label">
                                    <span class="label-text">Alt</span>
                                    <input v-model="keyboardShortcutSetting.alt" type="checkbox" class="toggle toggle-secondary" />
                                </div>
                                <div class="label">
                                    <span class="label-text">Key</span>
                                    <Select v-model="keyboardShortcutSetting.key">
                                        <SelectItem
                                            v-for="key in [
                                                'A',
                                                'B',
                                                'C',
                                                'D',
                                                'E',
                                                'F',
                                                'G',
                                                'H',
                                                'I',
                                                'J',
                                                'K',
                                                'L',
                                                'M',
                                                'N',
                                                'O',
                                                'P',
                                                'Q',
                                                'R',
                                                'S',
                                                'T',
                                                'U',
                                                'V',
                                                'W',
                                                'X',
                                                'Y',
                                                'Z',
                                                '0',
                                                '1',
                                                '2',
                                                '3',
                                                '4',
                                                '5',
                                                '6',
                                                '7',
                                                '8',
                                                '9',
                                                'F1',
                                                'F2',
                                                'F3',
                                                'F4',
                                                'F5',
                                                'F6',
                                                'F7',
                                                'F8',
                                                'F9',
                                                'F10',
                                                'F11',
                                                'F12',
                                                'Insert',
                                                'Delete',
                                                'Home',
                                                'End',
                                                'PgUp',
                                                'PgDown',
                                            ]"
                                            :key="key"
                                            :value="key"
                                            >{{ key }}</SelectItem
                                        >
                                    </Select>
                                </div>
                            </div>
                        </template>
                    </Popover>
                </div>
                <!-- 输入框 -->
                <RichInput
                    mode="html"
                    v-model="newMsgText"
                    @loadref="(r) => (input = r)"
                    @enter="sendMessage"
                    :placeholder="$t('chat.chatPlaceholder')"
                    class="flex-1 overflow-hidden"
                />
                <!-- 操作栏 -->
                <div class="flex p-2">
                    <div class="flex-1"></div>
                    <button class="btn btn-sm btn-primary px-6">{{ $t("chat.send") }}</button>
                </div>
            </form>
        </div>
        <div v-else-if="state === 'failed'" class="flex-1 flex flex-col gap-2 items-center justify-center">
            <div class="text-bold">{{ $t("chat.joinFailed") }}</div>
            <div class="text-sm text-base-content/50">{{ $t("chat.joinFailedTip") }}</div>
        </div>
        <div v-else class="flex-1 flex flex-col gap-2 items-center justify-center">
            <span class="loading loading-spinner loading-md"></span>
        </div>
    </div>
</template>
<style lang="less">
.safe-html {
    img {
        border-radius: 0.3rem;
    }
}
</style>
