<script lang="ts" setup>
import { ref } from "vue"
import { useUserStore } from "../mod/state/user"
import { nextTick } from "vue"
import { useTTS } from "../mod/api/tts"
import { useASR } from "../mod/api/asr"
import { useVAD } from "../mod/api/vad"
import { useLocalStorage } from "@vueuse/core"

let user = useUserStore()

let msgs = ref<{ content: string; url: string; createdAt: string; user: { id: string; name: string; qq: string } }[]>([])
let tts_text = ref("")
let tts_speed = ref(1)
let tts_engine = useLocalStorage("tts_engine", "GPTSoVITS")
let { tts, list: tts_engine_list, speaker: tts_voice, speakers: tts_list } = useTTS(tts_engine)
let { asr } = useASR()
let loading = ref(false)

function retractMessage(msg: { url: string }) {
    msgs.value = msgs.value.filter((item) => item.url !== msg.url)
}

async function sendMessage(e: Event) {
    if ((e as KeyboardEvent)?.shiftKey) {
        return
    }
    e.preventDefault()
    const content = tts_text.value
    if (!content) return
    tts_text.value = ""
    input.value!.focus()
    const index = msgs.value.push({
        content,
        url: "",
        createdAt: new Date().toLocaleString("zh-CN"),
        user: {
            id: user.id || "guest",
            name: user.name || "guest",
            qq: user.qq || "",
        },
    })
    await nextTick()
    el.value?.scrollTo({
        top: el.value.scrollHeight,
        left: 0,
        behavior: "smooth",
    })
    if (!tts.value) return
    msgs.value[index - 1].url = await tts.value.tts(content, { speaker: tts_voice.value, speed: tts_speed.value })
}
const el = ref<HTMLElement | null>(null)
const input = ref<HTMLDivElement | null>(null)
const inputForm = ref<HTMLDivElement | null>(null)
const { active: vadActive, speaking } = useVAD({
    nextWav: async (wav) => {
        if (!asr.value) return
        const res = await asr.value.asr(wav, { lang: "zh" })
        onVADMsg(res, wav)
    },
})
async function onVADMsg(text: string, blob: Blob) {
    msgs.value.push({
        content: text,
        url: URL.createObjectURL(blob),
        createdAt: new Date().toLocaleString("zh-CN"),
        user: {
            id: "unknown",
            name: "",
            qq: "3628855612",
        },
    })
    await nextTick()
    el.value?.scrollTo({
        top: el.value.scrollHeight,
        left: 0,
        behavior: "smooth",
    })
}
</script>

<template>
    <div class="w-full h-full bg-base-200/50 flex">
        <!-- 聊天窗口 -->
        <div class="flex-1 flex flex-col overflow-hidden">
            <!-- 消息列表 -->
            <ScrollArea @loadref="(r) => (el = r)" class="flex-1 overflow-hidden">
                <div class="flex w-full h-full flex-col gap-2 p-4">
                    <div class="group flex items-start gap-2" v-for="item in msgs" :key="item.url">
                        <div class="flex-1 flex items-start gap-2" :class="{ 'flex-row-reverse': user.id === item.user.id }">
                            <QQAvatar class="mt-2 size-8" :qq="item.user.qq" :name="item.user.name"></QQAvatar>

                            <ContextMenu class="flex items-start flex-col" :class="{ 'items-end': user.id === item.user.id }">
                                <div class="text-base-content/60 text-sm min-h-5">{{ item.user.name }}</div>

                                <div
                                    class="flex items-center gap-2"
                                    :class="{
                                        'flex-row-reverse': user.id === item.user.id,
                                    }"
                                >
                                    <div
                                        class="p-2 rounded-lg bg-base-100 select-text inline-flex text-sm max-w-80 overflow-hidden gap-2"
                                        :class="{ 'bg-primary text-base-100': user.id === item.user.id }"
                                    >
                                        {{ item.content }}
                                    </div>
                                    <AudioPlayButton v-if="item.url !== '-'" :autoplay="user.id === item.user.id" :src="item.url" />
                                </div>

                                <template #menu>
                                    <ContextMenuItem
                                        @click="retractMessage(item)"
                                        class="group text-sm p-2 leading-none text-base-content rounded flex items-center relative select-none outline-none data-[disabled]:text-base-content/60 data-[disabled]:pointer-events-none data-[highlighted]:bg-primary data-[highlighted]:text-base-100"
                                    >
                                        <Icon class="size-4 mr-2" icon="la:reply-solid" />
                                        {{ $t("chat.revert") }}
                                    </ContextMenuItem>
                                </template>
                            </ContextMenu>
                            <div class="flex-1"></div>
                            <div class="hidden group-hover:block p-1 text-xs text-base-content/60">{{ item.createdAt }}</div>
                        </div>
                    </div>
                </div>
            </ScrollArea>
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
                <!-- loading遮罩 -->
                <div v-if="loading" class="absolute top-0 left-0 bottom-0 right-0 cursor-progress z-100 flex justify-center items-center">
                    <span class="loading loading-spinner loading-md"></span>
                </div>
                <!-- 工具栏 -->
                <div class="flex-none p-1 px-2 border-t-[1px] border-base-300/50 flex items-center gap-2 pointer-events">
                    <span class="text-sm">引擎:</span>
                    <Select v-model="tts_engine">
                        <SelectItem v-for="item in tts_engine_list" :key="item.engine" :value="item.engine">{{ item.name }}</SelectItem>
                    </Select>
                    <span class="text-sm">角色:</span>
                    <Select v-model="tts_voice">
                        <SelectItem v-for="item in tts_list" :key="item" :value="item">{{ item }}</SelectItem>
                    </Select>
                    <span class="text-sm">速度({{ tts_speed }}):</span>
                    <div class="min-w-32 flex items-center">
                        <input
                            :value="tts_speed"
                            @input="tts_speed = +($event.target as HTMLInputElement)!.value"
                            type="range"
                            class="range range-secondary range-xs"
                            min="0.5"
                            max="2.0"
                            step="0.1"
                        />
                    </div>
                    <div
                        @click="vadActive = !vadActive"
                        class="btn btn-sm btn-square text-lg"
                        :class="{ 'text-primary': vadActive, 'text-secondary': speaking }"
                    >
                        <Icon icon="la:microphone-solid" />
                    </div>
                    <span class="text-sm" v-if="speaking">说话中...</span>
                </div>
                <!-- 输入框 -->
                <RichInput
                    mode="text"
                    v-model="tts_text"
                    @loadref="(r) => (input = r)"
                    @enter="sendMessage"
                    placeholder="输入内容"
                    class="flex-1 overflow-hidden pointer-events-auto"
                />
                <!-- 操作栏 -->
                <div class="flex p-2 pointer-events-auto">
                    <div class="flex-1"></div>
                    <button class="btn btn-sm btn-primary px-6" :disabled="loading">{{ $t("chat.send") }}</button>
                </div>
            </form>
        </div>
    </div>
</template>
