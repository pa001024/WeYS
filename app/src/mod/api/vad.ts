import { MicVAD } from "@ricky0123/vad-web"
import { onMounted, ref, watch } from "vue"
import { convertFloat32ArrayToWAV } from "../util/audio"

export interface VADOptions {
    autoStart?: boolean
    onActive?: () => void
    onInactive?: (audio: Float32Array) => any
    nextWav?: (wav: Blob) => any
}

export function useVAD(options?: VADOptions) {
    const { autoStart, onActive, onInactive, nextWav } = options || {}
    let vad: MicVAD
    let speaking = ref(false)
    let active = ref(autoStart || false)
    async function init() {
        vad = await MicVAD.new({
            workletURL: "/assets/vad.worklet.bundle.min.js",
            modelURL: "/assets/silero_vad.onnx",
            onSpeechStart: () => {
                speaking.value = true
                onActive && onActive()
            },
            onSpeechEnd: async (audio) => {
                speaking.value = false
                onInactive && onInactive(audio)
                nextWav && nextWav(convertFloat32ArrayToWAV(audio, 16000, 1))
            },
        })
    }
    onMounted(async () => {
        if (!autoStart) return
        init()
    })
    watch(active, async (v) => {
        if (v) {
            if (!vad) await init()
            vad?.start()
        } else {
            speaking.value = false
            vad?.pause()
        }
    })

    return { speaking, active }
}
