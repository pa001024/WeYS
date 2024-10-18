import Dexie from "dexie"
import type { TypedDB } from "dexie"
export const db = new Dexie("we-ys") as unknown as TypedDB<DB>

export interface User {
    id: string
    name: string
}

export interface GameAccount {
    id: number
    hash: string
    name: string
    lock: number
    uid: string // 米游社 uid
    usk: string
    usd: string
    login: string
    pwd: string
    token: string
}

export interface Account {
    id: number
    name: string
    uid: string // 米游社 uid
    gtoken: string // game token
    ctoken: string // cookie token
    mid: string // 米哈游 uid
    stoken: string // stoken v2
}

export interface Statistics {
    id: number
    date: string // fk
    userid: string // fk
    duration: string
}

export interface Device {
    id: number
    uuid: string // 设备标识，uuid，登录后建议不要修改
    type: string // 手机类型，默认 2 为安卓
    name: string // 手机型号，默认 Xiaomi 22011211C
    model: string // 手机型号，默认 22011211C
    version: string // 手机安卓版本，默认 13
    channel: string // 渠道，默认 miyousheluodi
}

export interface Room {
    id: number
    userId: string
    roomId: string
    readedCount: number
}

interface DB {
    users: User
    statistics: Statistics
    devices: Device
    accounts: Account
    gameAccounts: GameAccount
    rooms: Room
}

db.version(1).stores({
    users: "&id, sid, name",
    statistics: "++id, date, userid",
    devices: "++id, &uuid",
    accounts: "++id",
    gameAccounts: "++id, hash, lock",
    rooms: "++id, [userId+roomId]",
})
