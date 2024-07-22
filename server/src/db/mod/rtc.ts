import type { CreateMobius, Resolver } from "graphql-mobius"
import { Context } from "../yoga"
import { RtcClient } from "../../rt/pubsub"
import { id } from "../schema"

export const typeDefs = /* GraphQL */ `
    type Mutation {
        "发送信令"
        rtcSignal(roomId: String!, type: String!, from: String!, to: String!, body: String!): Boolean!
        "获取rtcId"
        rtcJoin(roomId: String!): RtcJoinResult
    }

    type Query {
        "获取当前房间的所有客户端"
        rtcClients(roomId: String!): [RtcClient!]!
    }
    type Subscription {
        "订阅新的客户端变更"
        newRtc(roomId: String!): RtcClient!
        "订阅新的信令事件"
        newRtcEvent(roomId: String!): RtcEvent!
    }

    type RtcJoinResult {
        id: String!
        end: Boolean!
        user: TinyUser!
        clients: [RtcClient!]!
    }

    type RtcClient {
        id: String!
        end: Boolean!
        user: TinyUser!
    }

    type TinyUser {
        id: String!
        name: String!
        qq: String
    }

    type RtcEvent {
        id: String!
        type: String!
        from: String!
        to: String!
        body: String!
    }
`

const rtcClientsMap = new Map<string, RtcClient[]>()

function addClient(id: string, roomId: string, user: { id: string; name: string; qq?: string }) {
    const clients = rtcClientsMap.get(roomId) || []

    const newRtc = {
        id,
        end: false,
        user: {
            id: user.id,
            name: user.name,
            qq: user.qq,
        },
    }
    rtcClientsMap.set(roomId, [...clients, newRtc])
    return newRtc
}

function removeClient(id: string, roomId: string, user: { id: string; name: string; qq?: string }) {
    const clients = rtcClientsMap.get(roomId) || []
    rtcClientsMap.set(
        roomId,
        clients.filter((c) => c.id !== id)
    )
    return {
        id,
        end: true,
        user: {
            id: user.id,
            name: user.name,
            qq: user.qq,
        },
    }
}

function getId(roomId: string, userId: string) {
    return rtcClientsMap.get(roomId)?.find((c) => c.user.id === userId)?.id
}

// 内存RTC信令服务器
export const resolvers = {
    Query: {
        rtcClients: async (parent, { roomId }, context, info: any) => {
            if (!context.user) return []
            return rtcClientsMap.get(roomId) || []
        },
    },
    Mutation: {
        rtcJoin: async (parent, { roomId }, { user, pubsub }, info) => {
            if (!user) return null
            return {
                id: getId(roomId, user.id),
                end: false,
                user: {
                    id: user.id,
                    name: user.name,
                    qq: user.qq,
                },
                clients: rtcClientsMap.get(roomId) || [],
            }
        },
        rtcSignal: async (parent, { roomId, type, from, to, body }, { user, pubsub }, info) => {
            if (!user) return false
            const clients = rtcClientsMap.get(roomId)
            const rtc = clients?.find((c) => c.id === from && c.user.id === user.id)
            pubsub.publish("newRtcEvent", roomId, { newRtcEvent: { id: id(), type, from, to, body } })
            if (rtc) return true
            return false
        },
    },
    Subscription: {
        newRtc: (parent, { roomId }, { user, pubsub, extra }, info) => {
            if (!user) throw new Error("need login")
            const socket = extra?.socket
            if (socket) {
                const newRtc = addClient(socket.data.id, roomId, user)
                pubsub.publish("newRtc", roomId, { newRtc })
                const oldclose = socket.data.close
                socket.data.close = async (ws) => {
                    oldclose?.(ws)
                    const newRtc = removeClient(socket.data.id, roomId, user)
                    pubsub.publish("newRtc", roomId, { newRtc })
                }
            }
            return pubsub.subscribe("newRtc", roomId)
        },
        newRtcEvent: (parent, { roomId }, { user, pubsub }, info) => {
            if (!user) throw new Error("need login")
            return pubsub.subscribe("newRtcEvent", roomId)
        },
    },
} satisfies Resolver<CreateMobius<typeof typeDefs>, Context>
