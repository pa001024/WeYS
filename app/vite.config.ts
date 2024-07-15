import { defineConfig } from "vite"
import vue from "@vitejs/plugin-vue"
import vueJsx from "@vitejs/plugin-vue-jsx"
import Component from "unplugin-vue-components/vite"
import RadixVueResolver from "radix-vue/resolver"
import { viteStaticCopy } from "vite-plugin-static-copy"

// https://vitejs.dev/config/
export default defineConfig(async () => ({
    plugins: [
        vue(),
        vueJsx(),
        Component({
            dts: "./src/components.d.ts",
            resolvers: [
                RadixVueResolver(),

                // RadixVueResolver({
                //   prefix: '' // use the prefix option to add Prefix to the imported components
                // })
            ],
        }),
        viteStaticCopy({
            targets: [
                {
                    src: "../node_modules/@ricky0123/vad-web/dist/vad.worklet.bundle.min.js",
                    dest: "./assets/",
                },
                {
                    src: "../node_modules/@ricky0123/vad-web/dist/silero_vad.onnx",
                    dest: "./assets/",
                },
                {
                    src: "../node_modules/onnxruntime-web/dist/*.wasm",
                    dest: "./",
                },
            ],
        }),
    ],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        watch: {
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
        },
    },
    optimizeDeps: {
        esbuildOptions: {
            target: "es2020",
        },
    },
    build: {
        // Tauri supports es2021
        target: ["es2020"],
        // don't minify for debug builds
        minify: !process.env.TAURI_DEBUG ? ("esbuild" as const) : false,
        // produce sourcemaps for debug builds
        sourcemap: !!process.env.TAURI_DEBUG,
    },
}))
