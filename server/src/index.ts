import { Elysia } from "elysia"
import { staticPlugin } from "@elysiajs/static"
import { cors } from "@elysiajs/cors"
import { yogaPlugin } from "./db"

const app = new Elysia()
    .use(staticPlugin({ prefix: "/", assets: "dist" }))
    .use(
        cors({
            // origin: "*",
            maxAge: 3600,
            allowedHeaders: '*',
            exposeHeaders: '*',
        })
    )
    .use(yogaPlugin())

app.listen(8887)
console.log(`🦊 Elysia is running at http://${app.server?.hostname}:${app.server?.port}`)
