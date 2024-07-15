import { Client, fetchExchange, subscriptionExchange, gql } from "@urql/vue"
import { useUserStore } from "../state/user"
import { SubscribePayload, createClient } from "graphql-ws"
import { offlineExchange } from "@urql/exchange-graphcache"
import { makeDefaultStorage } from "@urql/exchange-graphcache/default-storage"
import { nanoid } from "nanoid"
import { env } from "../../env"
// import schema from "../../schema.json"

const storage = makeDefaultStorage({
    idbName: "graphcache-v3", // The name of the IndexedDB database
    maxAge: 7, // The maximum age of the persisted data in days
})

const cacheExchange = offlineExchange({
    resolvers: {
        Query: {
            roomMsgCount: (data, args, cache, _info) => {
                const roomCache = cache.readFragment<{ id: string; msgCount?: number }>(
                    gql`
                        fragment _ on Room {
                            id
                            msgCount
                        }
                    `,
                    { id: args.roomId as any }
                )
                return {
                    __typename: "Room",
                    id: args.roomId,
                    msgCount: roomCache?.msgCount || 0,
                }
            },
        },
    },
    // schema,
    storage,
    updates: {
        Mutation: {},
        Subscription: {
            newMessage(result: any, args, cache, _info) {
                const msg = result.newMessage
                const roomCache = cache.readFragment<{ id: string; msgCount?: number }>(
                    gql`
                        fragment _ on Room {
                            id
                            msgCount
                        }
                    `,
                    { id: args.roomId as any }
                )
                const count = (roomCache?.msgCount || 0) + 1
                // 写入房间最新消息和消息数量
                cache.writeFragment(
                    gql`
                        fragment _ on Room {
                            id
                            msgCount
                            lastMsg {
                                id
                                content
                                user {
                                    id
                                    name
                                    qq
                                }
                            }
                        }
                    `,
                    {
                        id: args.roomId,
                        msgCount: count,
                        lastMsg: {
                            __typename: "Msg",
                            id: msg.id,
                            content: args.content,
                            createdAt: msg.createdAt,
                            user: {
                                __typename: "User",
                                id: msg.user.id,
                                name: msg.user.name,
                                qq: msg.user.qq,
                            },
                        },
                    }
                )
                // 更新房间消息缓存
                cache.updateQuery(
                    {
                        query: gql`
                            query ($roomId: String!, $limit: Int, $offset: Int) {
                                msgs(roomId: $roomId, limit: $limit, offset: $offset) {
                                    id
                                    edited
                                    content
                                    createdAt
                                    user {
                                        id
                                        name
                                        qq
                                    }
                                }
                            }
                        `,
                        variables: { roomId: args.roomId, limit: 20, offset: count - (count % 20 || 20) },
                    },
                    (data) => {
                        if (!data) return { msgs: [msg] }
                        data.msgs.push(msg)
                        // data.msgs.shift()
                        return data
                    }
                )
            },
            msgEdited(result: any, args, cache, _info) {
                const fragment = gql`
                    fragment _ on Msg {
                        id
                        content
                        edited
                    }
                `

                const msg = result.msgEdited
                cache.writeFragment(fragment, {
                    id: msg.id,
                    content: msg.content,
                    edited: msg.edited,
                })
            },
        },
    },
    optimistic: {},
})

export const gqClient = (function () {
    const url = `${env.endpoint}/graphql`
    const ws = createClient({
        url: url.replace("http", "ws"),
        connectionParams: () => {
            const token = useUserStore().token
            return {
                token,
            }
        },
        generateID: () => nanoid(),
    })
    return new Client({
        url,
        exchanges: [
            cacheExchange,
            fetchExchange,
            subscriptionExchange({
                forwardSubscription(operation) {
                    return {
                        subscribe: (sink) => {
                            const dispose = ws.subscribe(operation as SubscribePayload, sink)
                            return {
                                unsubscribe: dispose,
                            }
                        },
                    }
                },
            }),
        ],
        fetchOptions: () => {
            const token = useUserStore().token
            return {
                headers: { token },
            }
        },
    })
})()
