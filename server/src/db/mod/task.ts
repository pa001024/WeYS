import type { CreateMobius, Resolver } from "graphql-mobius"
import { and, eq, isNull } from "drizzle-orm"
import { Context } from "../yoga"
import { db, schema } from ".."
import { now } from "../schema"

export const typeDefs = /* GraphQL */ `
    type Query {
        "获取任务"
        tasks(roomId: String!, limit: Int, offset: Int): [Task!]!
        "获取进行中的任务"
        doingTasks(roomId: String!): [Task!]!
    }
    type Mutation {
        "添加任务"
        addTask(roomId: String!, name: String!, maxUser: Int, maxAge: Int, desc: String): Task
        "异步添加任务等待结束"
        addTaskAsync(roomId: String!, name: String!, maxUser: Int, maxAge: Int, desc: String): Task
        "加入任务"
        joinTask(taskId: String!): Boolean!
        "结束任务"
        endTask(taskId: String!): Boolean!
    }
    type Subscription {
        "订阅新任务"
        newTask(roomId: String!): Task!
        "订阅任务更新"
        updateTask(roomId: String!): Task!
    }

    type Task {
        id: String!
        name: String!
        desc: String
        maxUser: Int!
        maxAge: Int
        userList: [String!]!
        startTime: String
        endTime: String
        roomId: String!
        userId: String!
        createdAt: String
        updateAt: String

        user: User!
    }
`

export const resolvers = {
    Query: {
        tasks: async (parent, { roomId, limit, offset }, context, info) => {
            if (!context.user) return []
            const last = await db.query.tasks.findMany({
                with: {
                    user: true,
                },
                where: eq(schema.tasks.roomId, roomId),
                limit,
                offset,
            })
            return last
        },
        doingTasks: async (parent, { roomId }, context, info) => {
            if (!context.user) return []
            const last = await db.query.tasks.findMany({
                with: {
                    user: true,
                },
                where: and(eq(schema.tasks.roomId, roomId), isNull(schema.tasks.endTime)),
            })
            return last
        },
    },
    Mutation: {
        addTask: async (parent, { roomId, name, maxUser, maxAge, desc }, { user, pubsub }, info) => {
            if (!user) return null
            const res = (
                await db
                    .insert(schema.tasks)
                    .values({
                        name,
                        maxUser: maxUser || 3,
                        maxAge,
                        desc,
                        roomId,
                        userId: user.id,
                        userList: [],
                    })
                    .returning()
            )[0]
            if (res) {
                const task = await db.query.tasks.findFirst({
                    with: { user: true },
                    where: eq(schema.tasks.id, res.id),
                })
                if (task) {
                    pubsub.publish("newTask", roomId, { newTask: task })
                    if (task.maxAge) {
                        setTimeout(async () => {
                            const taskNeedEnd = await db.query.tasks.findFirst({
                                with: { user: true },
                                where: eq(schema.tasks.id, res.id),
                            })
                            if (taskNeedEnd && !taskNeedEnd.endTime) {
                                taskNeedEnd.endTime = now()
                                await db.update(schema.tasks).set({ endTime: taskNeedEnd.endTime }).where(eq(schema.tasks.id, taskNeedEnd.id))
                                pubsub.publish("updateTask", roomId, { updateTask: taskNeedEnd })
                            }
                        }, task.maxAge * 1000)
                    }
                    return task
                }
            }
            return null
        },
        addTaskAsync: async (parent, { roomId, name, maxUser, maxAge, desc }, { user, pubsub }, info) => {
            if (!user) return null
            const res = (
                await db
                    .insert(schema.tasks)
                    .values({
                        name,
                        maxUser: maxUser || 3,
                        maxAge,
                        desc,
                        roomId,
                        userId: user.id,
                        userList: [],
                    })
                    .returning()
            )[0]
            if (res) {
                const task = await db.query.tasks.findFirst({
                    with: { user: true },
                    where: eq(schema.tasks.id, res.id),
                })
                if (task) {
                    pubsub.publish("newTask", roomId, { newTask: task })
                    const idle = pubsub.subscribe("updateTask", roomId)
                    while (true) {
                        const message = await idle.next()
                        if (message.value.updateTask.id === task.id) {
                            const task = await db.query.tasks.findFirst({
                                with: { user: true },
                                where: eq(schema.tasks.id, res.id),
                            })
                            return task
                        }
                    }
                }
            }
            return null
        },
        joinTask: async (parent, { taskId }, { user, pubsub }, info) => {
            if (!user) return false
            const task = await db.query.tasks.findFirst({
                with: { user: true },
                where: eq(schema.tasks.id, taskId),
            })
            if (!task) return false
            if (task.userList.includes(user.id) || task.endTime) {
                return false
            }
            task.userList.push(user.id)
            task.startTime = task.startTime || now()
            task.endTime = task.userList.length >= task.maxUser ? now() : null
            await db
                .update(schema.tasks)
                .set({
                    userList: task.userList,
                    startTime: task.startTime,
                    endTime: task.endTime,
                })
                .where(eq(schema.tasks.id, taskId))
            pubsub.publish("updateTask", task.roomId, { updateTask: task })
            return true
        },
        endTask: async (parent, { taskId }, { user, pubsub }, info) => {
            if (!user) return false
            const task = await db.query.tasks.findFirst({
                with: { user: true },
                where: eq(schema.tasks.id, taskId),
            })
            if (!task) return false
            if (task.endTime) {
                return false
            }
            task.startTime = task.startTime || now()
            task.endTime = now()
            await db
                .update(schema.tasks)
                .set({
                    startTime: task.startTime,
                    endTime: task.endTime,
                })
                .where(eq(schema.tasks.id, taskId))
            pubsub.publish("updateTask", task.roomId, { updateTask: task })
            return true
        },
    },
    Subscription: {
        newTask: async (parent, { roomId }, { user, pubsub }, info) => {
            if (!user) throw new Error("need login")
            return pubsub.subscribe("newTask", roomId)
        },
        updateTask: async (parent, { roomId }, { user, pubsub }, info) => {
            if (!user) throw new Error("need login")
            return pubsub.subscribe("updateTask", roomId)
        },
    },
} satisfies Resolver<MsgGQL, Context>

export type MsgGQL = CreateMobius<typeof typeDefs>
