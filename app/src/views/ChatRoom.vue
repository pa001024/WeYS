<script lang="ts" setup>
import { useTimestamp } from "@vueuse/core"
import { useScroll } from "@vueuse/core"
import { computed, onMounted, ref, watch } from "vue"
import { isImage, sanitizeHTML } from "../mod/util/html"
import { gql, useQuery, useSubscription } from "@urql/vue"
import { useRoute } from "vue-router"
import { useEditMessageMutation, useSendMessageMutation } from "../mod/api/mutation"
import { useUserStore } from "../mod/state/user"

const route = useRoute()
const roomId = computed(() => route.params.room as string)
const user = useUserStore()

const el = ref<HTMLElement | null>(null)

const { arrivedState } = useScroll(el, { offset: { left: 0, top: 20, right: 0, bottom: 200 } })

const { data } = useQuery<{ roomMsgCount: { msgCount: number } }>({
    query: gql`
        query ($roomId: String!) {
            roomMsgCount(roomId: $roomId) {
                id
                msgCount
            }
        }
    `,
    variables: { roomId },
    requestPolicy: "cache-only",
})
const msgCount = computed(() => data.value?.roomMsgCount?.msgCount || 0)
watch(msgCount, (count) => {
    if (msgCount.value > 0) {
        user.setRoomReadedCount(roomId.value, count)
    }
})
onMounted(() => {
    if (msgCount.value > 0) user.setRoomReadedCount(roomId.value, msgCount.value)
})

const Query = /* GraphQL */ `
    query ($roomId: String!, $limit: Int, $offset: Int) {
        msgs(roomId: $roomId, limit: $limit, offset: $offset) {
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
`
const variables = computed(() => ({ roomId: roomId.value }))

type Msg = {
    id: string
    edited: number
    content: string
    createdAt: string
    user: { id: string; name: string; qq: number }
}

useSubscription<{ newMessage: Msg; msgEdited: Msg }, Msg[]>(
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
        return []
    }
)

useSubscription<{ newMessage: Msg; msgEdited: Msg }, Msg[]>({
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

const doSendMessage = useSendMessageMutation()
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
    await doSendMessage({ content, roomId: roomId.value })
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

const editId = ref("")
const editInput = ref<HTMLDivElement[] | null>(null)
const doEditMessage = useEditMessageMutation()
async function editMessage(msgId: string, content: string) {
    await doEditMessage({ content, msgId })
}
const retractCache = new WeakMap()
async function retractMessage(msg: Msg) {
    retractCache.set(msg, msg.content)
    await editMessage(msg.id, "")
    msg.content = ""
}
async function restoreMessage(msg: Msg) {
    const content = retractCache.get(msg)
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
            await editMessage(editId.value, newVal)
            msg.content = newVal
            msg.edited = 1
            editId.value = ""
        }
    }
}
</script>

<template>
    <div class="w-full h-full bg-base-200/50 flex">
        <!-- 聊天窗口 -->
        <div class="flex-1 flex flex-col overflow-hidden">
            <GQAutoPage
                v-if="msgCount"
                @loadref="(r) => (el = r)"
                direction="top"
                class="flex-1 overflow-hidden"
                innerClass="flex w-full h-full flex-col gap-2 p-4"
                :limit="20"
                :offset="msgCount"
                :query="Query"
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
                                    <Icon class="size-4 mr-2" icon="la:eye" />
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
                                    @click="retractMessage(item)"
                                    class="group text-sm p-2 leading-none text-base-content rounded flex items-center relative select-none outline-none data-[disabled]:text-base-content/60 data-[disabled]:pointer-events-none data-[highlighted]:bg-primary data-[highlighted]:text-base-100"
                                >
                                    <Icon class="size-4 mr-2" icon="la:reply-solid" />
                                    {{ $t("chat.revert") }}
                                </ContextMenuItem>
                                <ContextMenuItem
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
                    <div class="flex btn btn-primary" @click="doSendMessage({ content: $t('chat.hello'), roomId })">
                        {{ $t("chat.sayHello") }}
                    </div>
                </div>
            </div>
            <!-- 分割线 -->
            <div class="flex-none w-full relative">
                <div
                    class="w-full absolute -mt-[3px] h-[6px] cursor-ns-resize z-100"
                    v-h-resize-for="{ el: inputForm, min: 120, max: 400 }"
                ></div>
            </div>
            <!-- 输入 -->
            <form
                class="h-44 flex flex-col relative border-t-[1px] border-base-300/50 pointer-events-none"
                ref="inputForm"
                @submit="sendMessage"
            >
                <!-- 工具栏 -->
                <div class="flex-none p-1 px-2 border-t-[1px] border-base-300/50 flex items-center gap-2">
                    <!-- 表情 -->
                    <Popover side="top">
                        <div class="btn btn-sm btn-square text-2xl hover:text-primary pointer-events-auto">
                            <Icon icon="la:smile" />
                        </div>
                        <template #content>
                            <EmojiSelect @select="insertEmoji" />
                        </template>
                    </Popover>
                </div>
                <!-- 输入框 -->
                <RichInput
                    mode="text"
                    v-model="newMsgText"
                    @loadref="(r) => (input = r)"
                    @enter="sendMessage"
                    placeholder="输入聊天内容"
                    class="flex-1 overflow-hidden pointer-events-auto"
                />
                <!-- 操作栏 -->
                <div class="flex p-2 pointer-events-auto">
                    <div class="flex-1"></div>
                    <button class="btn btn-sm btn-primary px-6">{{ $t("chat.send") }}</button>
                </div>
            </form>
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
