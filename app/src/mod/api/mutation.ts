import { AnyVariables, gql, OperationContext } from "@urql/vue"
import { gqClient } from "./graphql"

function extractType<T extends string>(gqlQuery: T) {
    const match = gqlQuery.match(/mutation[\s\S]*?\s(\w+?)\s*\(/m)
    if (match) {
        return match[1]
    }
    return ""
}

function namedMutation<R = { id: string }, G extends string = string>(gqlQuery: G) {
    const name = extractType(gqlQuery)
    const query = gql(gqlQuery)
    return async (variables?: AnyVariables, context?: Partial<OperationContext> | undefined) => {
        const raw = await gqClient.mutation(query, variables, context).toPromise()
        return raw?.data?.[name] as R
    }
}

export const createRoomMutation = namedMutation(/* GraphQL */ `
    mutation ($name: String!, $type: String!) {
        createRoom(data: { name: $name, type: $type }) {
            id
        }
    }
`)

export const sendMessageMutation = namedMutation(/* GraphQL */ `
    mutation ($content: String!, $roomId: String!) {
        sendMessage(content: $content, roomId: $roomId) {
            id
        }
    }
`)

export const editMessageMutation = namedMutation(/* GraphQL */ `
    mutation ($content: String!, $msgId: String!) {
        editMessage(content: $content, msgId: $msgId) {
            id
        }
    }
`)

export const deleteRoomMutation = namedMutation<boolean>(/* GraphQL */ `
    mutation ($id: String!) {
        deleteRoom(id: $id)
    }
`)

export const loginMutation = namedMutation<{
    success: boolean
    message: string
    token: string
    user: {
        id: string
        name: string
        qq: string
    }
}>(/* GraphQL */ `
    mutation ($email: String!, $password: String!) {
        login(password: $password, email: $email) {
            success
            message
            token
            user {
                id
                name
                qq
            }
        }
    }
`)

export const registerMutation = namedMutation<{
    success: boolean
    message: string
    token: string
    user: {
        id: string
        name: string
        qq: string
    }
}>(/* GraphQL */ `
    mutation ($name: String!, $qq: String!, $email: String!, $password: String!) {
        register(name: $name, qq: $qq, email: $email, password: $password) {
            success
            message
            token
            user {
                id
                name
                qq
            }
        }
    }
`)

export const guestMutation = namedMutation<{
    success: boolean
    message: string
    token: string
    user: {
        id: string
        name: string
        qq: string
    }
}>(/* GraphQL */ `
    mutation ($name: String!, $qq: String!) {
        guest(name: $name, qq: $qq) {
            success
            message
            token
            user {
                id
                name
                qq
            }
        }
    }
`)

export const updatePasswordMutation = namedMutation<{
    success: boolean
    token: string
}>(/* GraphQL */ `
    mutation ($old_password: String!, $new_password: String!) {
        updatePassword(old_password: $old_password, new_password: $new_password) {
            success
            token
        }
    }
`)

export const updateUserMetaMutation = namedMutation<{
    success: boolean
    message: string
    token: string
    user: {
        id: string
        name: string
        qq: string
    }
}>(/* GraphQL */ `
    mutation ($name: String, $qq: String) {
        updateUserMeta(data: { name: $name, qq: $qq }) {
            success
            message
            token
            user {
                id
                name
                qq
            }
        }
    }
`)

export const addTaskMutation = namedMutation(/* GraphQL */ `
    mutation ($roomId: String!, $name: String!, $maxUser: Int, $desc: String) {
        addTask(roomId: $roomId, name: $name, maxUser: $maxUser, desc: $desc) {
            id
        }
    }
`)

export const addTaskAsyncMutation = namedMutation(/* GraphQL */ `
    mutation ($roomId: String!, $name: String!, $maxUser: Int, $desc: String) {
        addTaskAsync(roomId: $roomId, name: $name, maxUser: $maxUser, desc: $desc) {
            id
        }
    }
`)

export const addTaskEndAsyncMutation = namedMutation(/* GraphQL */ `
    mutation ($roomId: String!, $name: String!, $maxUser: Int, $desc: String) {
        addTaskEndAsync(roomId: $roomId, name: $name, maxUser: $maxUser, desc: $desc) {
            id
        }
    }
`)

export const endTaskMutation = namedMutation<boolean>(/* GraphQL */ `
    mutation ($taskId: String!) {
        endTask(taskId: $taskId)
    }
`)

export const joinTaskMutation = namedMutation<boolean>(/* GraphQL */ `
    mutation ($taskId: String!) {
        joinTask(taskId: $taskId)
    }
`)

export const rtcSignalMutation = namedMutation<boolean>(/* GraphQL */ `
    mutation ($roomId: String!, $type: String!, $from: String!, $to: String!, $body: String!) {
        rtcSignal(roomId: $roomId, type: $type, from: $from, to: $to, body: $body)
    }
`)

export const rtcJoinMutation = namedMutation<{
    id: string
    end: boolean
    user: {
        id: string
        name: string
        qq: string
    }
    clients: {
        id: string
        user: {
            id: string
            name: string
            qq: string
        }
    }[]
}>(/* GraphQL */ `
    mutation ($roomId: String!) {
        rtcJoin(roomId: $roomId) {
            id
            end
            user {
                id
                name
                qq
            }
            clients {
                id
                user {
                    id
                    name
                    qq
                }
            }
        }
    }
`)

export interface RoomInfo {
    id: string
    name: string
    type: string
    maxUsers: number
    createdAt: string
    updateAt: string
    msgCount: number
    onlineUsers: {
        id: string
        name: string
        qq: string
    }[]
}

export const joinRoomMutation = namedMutation<RoomInfo>(/* GraphQL */ `
    mutation ($id: String!) {
        joinRoom(id: $id) {
            id
            name
            type
            msgCount
            onlineUsers {
                id
                name
                qq
            }
        }
    }
`)
