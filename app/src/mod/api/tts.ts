import { MaybeRef, nextTick, onMounted, ref, watch } from "vue"
import { GradioClient } from "./gradio"
import { useLocalStorage } from "@vueuse/core"

export enum Engine {
    GPTSoVITS = "GPTSoVITS",
    CosyVoice = "CosyVoice",
}

export interface TTSEngine {
    init(): Promise<void>
    listSpeakers(): Promise<string[]>
    tts(text: string, options: { speaker?: string; speed?: number }): Promise<string>
}

export class TTSEngineManager {
    static list = [
        { name: "GPTSoVITS", engine: Engine.GPTSoVITS },
        { name: "CosyVoice", engine: Engine.CosyVoice },
    ]
    static engines: Record<string, TTSEngine> = {}
    static async getEngine(engine: string) {
        switch (engine) {
            case Engine.GPTSoVITS:
                if (!this.engines.GPTSOVITS) this.engines.GPTSOVITS = await GPTSoVITS.create("http://127.0.0.1:5000/")
                return this.engines.GPTSOVITS

            case Engine.CosyVoice:
                if (!this.engines.CosyVoice) this.engines.CosyVoice = await CosyVoice.create("http://127.0.0.1:9880/")
                return this.engines.CosyVoice
        }
    }
}

export class CosyVoice implements TTSEngine {
    constructor(public url: string) {
        if (this.url.endsWith("/")) {
            this.url = this.url.slice(0, -1)
        }
    }

    static async create(url: string) {
        const c = new this(url)
        await c.init()
        return c
    }

    async init() {}

    async listSpeakers() {
        const list = await fetch(this.url + "/speakers", {})
        return (await list.json()) as string[]
    }

    async tts(text: string, options: { speaker?: string; speed: number }) {
        const res = await fetch(this.url + "/tts", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                text: text,
                speaker: options.speaker,
                speed: options.speed,
            }),
        })
        return URL.createObjectURL(await res.blob())
    }
}

export class CosyVoiceGradio implements TTSEngine {
    client!: GradioClient
    constructor(public url: string) {
        if (this.url.endsWith("/")) {
            this.url = this.url.slice(0, -1)
        }
    }

    static async create(url: string) {
        const c = new this(url)
        await c.init()
        return c
    }

    async init() {
        this.client = await GradioClient.connect(this.url)
    }

    async listSpeakers() {
        const list = await this.client.call<{ choices: [string, string][] }>("/refresh_choices", {})
        return list.choices.map((v) => v[0])
    }

    async tts(text: string, options: { speaker?: string }) {
        const res = await this.client.call("/generate_audio", {
            tts_text: text,
            mode_checkbox_group: "预训练音色",
            sft_dropdown: "中文女",
            seed: 0,
            new_dropdown: options?.speaker,
        })
        return res
    }
}

export class GPTSoVITS implements TTSEngine {
    constructor(public url: string) {
        if (this.url.endsWith("/")) {
            this.url = this.url.slice(0, -1)
        }
    }

    static async create(url: string) {
        const c = new this(url)
        await c.init()
        return c
    }

    async init() {}

    async listSpeakers() {
        const res = await fetch(this.url + "/character_list")
        return Object.keys(await res.json())
    }

    async tts(text: string, options: { speaker?: string }) {
        const url = new URL(this.url + "/tts")
        if (options?.speaker) url.searchParams.append("cha_name", options.speaker)
        if (text) url.searchParams.append("text", text)
        return URL.createObjectURL(await (await fetch(url.toString())).blob())
    }
}

export function useTTS(engine?: MaybeRef<string>) {
    let tts = ref<TTSEngine>()
    let loading = ref(false)
    let speakers = ref<string[]>([])
    let speaker = useLocalStorage("tts-speaker", "")

    async function init() {
        await nextTick()
        loading.value = true
        tts.value = await TTSEngineManager.getEngine((typeof engine === "string" ? engine : engine?.value) || Engine.GPTSoVITS)
        speakers.value = (await tts.value?.listSpeakers()) || []
        speaker.value = speakers.value.includes(speaker.value) ? speaker.value : speakers.value[0] || ""
        loading.value = false
    }
    onMounted(init)
    if (typeof engine === "object") watch(engine, init)
    return { tts, loading, speakers, speaker, list: TTSEngineManager.list }
}
