import { MaybeRef, nextTick, onMounted, ref, watch } from "vue"
import { GradioClient } from "./gradio"

export enum Engine {
    SenseVoice = "SenseVoice",
    Whisper = "Whisper",
}

export interface ASREngine {
    init(): Promise<void>
    asr(wav: Blob, options?: { lang?: string }): Promise<string>
}

export class ASREngineManager {
    static list = [
        { name: "SenseVoice", engine: Engine.SenseVoice },
        { name: "Whisper", engine: Engine.Whisper },
    ]
    static engines: Record<string, ASREngine> = {}
    constructor() {}
    static async getEngine(engine: string) {
        switch (engine) {
            case Engine.SenseVoice:
                if (!this.engines.SenseVoice) this.engines.SenseVoice = await SenseVoice.create("http://127.0.0.1:7860/")
                return this.engines.SenseVoice

            case Engine.Whisper:
                throw new Error("not impl")
        }
    }
}

export class SenseVoice implements ASREngine {
    client!: GradioClient
    constructor(public url: string) {}

    static async create(url: string) {
        const c = new this(url)
        await c.init()
        return c
    }

    async init() {
        this.client = await GradioClient.connect(this.url)
    }

    async asr(wav: Blob, options?: { lang?: string }) {
        const res = await this.client.call("/model_inference", {
            input_wav: { path: await this.client.upload(wav) },
            language: options?.lang || "auto",
        })
        return res
    }
}

export function useASR(engine?: MaybeRef<string>) {
    let asr = ref<ASREngine>()
    let loading = ref(false)

    async function init() {
        await nextTick()
        loading.value = true
        asr.value = await ASREngineManager.getEngine((typeof engine === "string" ? engine : engine?.value) || Engine.SenseVoice)
        loading.value = false
    }
    onMounted(init)
    if (typeof engine === "object") watch(engine, init)
    return { asr, loading, list: ASREngineManager.list }
}
