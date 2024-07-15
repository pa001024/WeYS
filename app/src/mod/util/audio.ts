// Returns Uint8Array of WAV bytes
function getWavBytes(
    buffer: ArrayBufferLike,
    options: { isFloat?: boolean; numChannels?: number; sampleRate?: number; numFrames?: number }
) {
    const type = options.isFloat ? Float32Array : Uint16Array
    const numFrames = buffer.byteLength / type.BYTES_PER_ELEMENT

    const headerBytes = getWavHeader(Object.assign({}, options, { numFrames }))
    const wavBytes = new Uint8Array(headerBytes.length + buffer.byteLength)

    // prepend header, then add pcmBytes
    wavBytes.set(headerBytes, 0)
    wavBytes.set(new Uint8Array(buffer), headerBytes.length)

    return wavBytes
}

// adapted from https://gist.github.com/also/900023
// returns Uint8Array of WAV header bytes
function getWavHeader(options: { numFrames: number; numChannels?: number; sampleRate?: number; isFloat?: boolean }) {
    const numFrames = options.numFrames
    const numChannels = options.numChannels || 2
    const sampleRate = options.sampleRate || 44100
    const bytesPerSample = options.isFloat ? 4 : 2
    const format = options.isFloat ? 3 : 1

    const blockAlign = numChannels * bytesPerSample
    const byteRate = sampleRate * blockAlign
    const dataSize = numFrames * blockAlign

    const buffer = new ArrayBuffer(44)
    const dv = new DataView(buffer)

    let p = 0

    function writeString(s: string) {
        for (let i = 0; i < s.length; i++) {
            dv.setUint8(p + i, s.charCodeAt(i))
        }
        p += s.length
    }

    function writeUint32(d: number) {
        dv.setUint32(p, d, true)
        p += 4
    }

    function writeUint16(d: number) {
        dv.setUint16(p, d, true)
        p += 2
    }

    writeString("RIFF") // ChunkID
    writeUint32(dataSize + 36) // ChunkSize
    writeString("WAVE") // Format
    writeString("fmt ") // Subchunk1ID
    writeUint32(16) // Subchunk1Size
    writeUint16(format) // AudioFormat
    writeUint16(numChannels) // NumChannels
    writeUint32(sampleRate) // SampleRate
    writeUint32(byteRate) // ByteRate
    writeUint16(blockAlign) // BlockAlign
    writeUint16(bytesPerSample * 8) // BitsPerSample
    writeString("data") // Subchunk2ID
    writeUint32(dataSize) // Subchunk2Size

    return new Uint8Array(buffer)
}

export function convertFloat32ArrayToWAV(float32Array: Float32Array, sampleRate: number, numChannels: number) {
    const wavBytes = getWavBytes(float32Array.buffer, {
        isFloat: true, // floating point or 16-bit integer
        numChannels,
        sampleRate,
    })
    const wavBlob = new Blob([wavBytes], { type: "audio/wav" })

    return wavBlob
}

// Example usage:
// Assuming you have a Float32Array called audioData and a sampleRate
// const audioData = ...; // Your Float32Array audio data
// const sampleRate = 44100; // Sample rate in Hz
// const numberOfChannels = 2; // For stereo audio

// const wavBlob = convertFloat32
