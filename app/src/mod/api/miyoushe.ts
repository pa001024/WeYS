import md5 from "crypto-js/md5"
import { fetch } from "@tauri-apps/plugin-http"
const API_BASE_HK4E = "https://hk4e-sdk.mihoyo.com/hk4e_cn/combo/panda"
const API_BASE_Takumi = "https://api-takumi.mihoyo.com"
const HK4E = apiBase(API_BASE_HK4E)
const Takumi = apiBase(API_BASE_Takumi)

interface GetSTokenByGTokenData {
    token: {
        token_type: string
        token: string
    }
    user_info: {
        aid: string
        mid: string
    }
}

export class Signing {
    // 米游社版本 2.71.1
    static readonly version = "2.71.1"
    static readonly K2 = "rtvTthKxEyreVXQCnhluFgLXPOFKPHlA"
    static readonly LK2 = "EJncUPGnOHajenjLhBOsdpwEMZmiCmQX"
    static readonly SALT_4X = "xV8v4Qu54lUKrEYFZkJhB8cuOh9Asafs"
    static readonly SALT_6X = "t0qEgfub6cvueAPgR5m9aQWWVciEer7v"
    static readonly SALT_PROD = "JwYDpKvLj6MrMqqYU6jTKF17KNO2PXoS"

    /**
     * 在请求头中的x-rpc-client_type为5时使用。
     * @param query "server=cn_gf01&role_id=123456789"
     * @param body
     */
    static ds2(query: string, body: any) {
        if (typeof body !== "string") body = JSON.stringify(body)
        query = query.split("&").sort().join("&")
        const t = Math.floor(Date.now() / 1000)
        let r = Math.floor(Math.random() * 100001 + 100000)
        if (r == 100000) {
            r = 642367
        }
        const main = `salt=${this.SALT_4X}&t=${t}&r=${r}&b=${body}&q=${query}`
        const ds = md5(main)
        return `${t},${r},${ds}`
    }

    /**
     * 在请求头中的x-rpc-client_type为5时使用。
     */
    static ds1() {
        const lettersAndNumbers = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        const t = Math.floor(Date.now() / 1000)
        let r = ""
        for (let i = 0; i < 6; i++) {
            r += lettersAndNumbers[Math.floor(Math.random() * lettersAndNumbers.length)]
        }
        const main = `salt=${this.K2}&t=${t}&r=${r}`
        const ds = md5(main)
        return `${t},${r},${ds}`
    }
}

export enum AppID {
    HI3 = "1", // 《崩坏3》
    TT = "2", // 《未定事件簿》
    GI = "4", // 《原神》
    PA = "5", // 平台应用
    HI2 = "7", // 《崩坏学园2》
    HIS = "8", // 《崩坏：星穹铁道》
    YUN = "9", // 云游戏
    NNN = "10", // 3NNN
    PJSH = "11", // PJSH
    ZZZ = "12", // 《绝区零》
    HYG = "13", // HYG
}

export class MiyousheAPI {
    ticket: string | null = null
    constructor(public deviceId: string) {}
    /**
     * 生成二维码(国服)
     * @returns 
     * ```json
     * {
     *   "retcode": 0,
     *   "message": "OK",
     *   "data": {
     *     "url": "https://user.mihoyo.com/qr_code_in_game.html?app_id=7\u0026app_name=%E5%B4%A9%E5%9D%8F%E5%AD%A6%E5%9B%AD2&bbs=false\u0026biz_key=bh2_cn\u0026expire=1687002702\u0026ticket=648706ceff80ee663845a13d"
     *   }
     * }
    ```
     */
    async genQRCode() {
        const res = await HK4E.post<{ data: { url: string } }>("/qrcode/fetch", {
            app_id: AppID.GI,
            device: this.deviceId,
        })
        const url = new URL(res.data?.url)
        this.ticket = url.searchParams.get("ticket")
        return this.ticket
    }

    /**
     * 查询二维码扫描状态
     * @returns
     * ```json
     * {
     *  "retcode": 0,
     *  "message": "OK",
     *  "data": {
     *     "stat": "Confirmed", // Init | Scanned | ExpiredCode | Confirmed
     *         "payload": {
     *             "proto": "Account",
     *             "raw": "{\"uid\":\"317832114\",\"token\":\"***\"}",
     *             "ext": ""
     *         }
     *     }
     *  }
     *  ```
     */
    waitQRCode() {
        return HK4E.post("/qrcode/query", {
            app_id: AppID.GI,
            device: this.deviceId,
        })
    }

    /** get stoken v2 by game token */
    getSTokenByGToken(account: any) {
        return Takumi.post<GetSTokenByGTokenData>("/account/ma-cn-session/app/getTokenByGameToken", {
            account_id: account.uid,
            game_token: account.game_token,
        })
    }

    getCTokenBySToken() {
        return Takumi.post<{
            uid: string
            cookie_token: string
        }>("/auth/api/getCookieAccountInfoBySToken", {})
    }

    getLTokenBySToken() {}
}

async function post<T = {}>(url: string, data: any, headers: any = {}): Promise<T> {
    const res = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Accept: "application/json",
            ...headers,
        },
        body: JSON.stringify(data),
    })
    return await res.json()
}

function apiBase(base: string) {
    return {
        post: <T>(url: string, data: any, headers: any = {}) => post<T>(base + url, data, headers),
    }
}
