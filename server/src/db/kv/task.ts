export interface OnlineStatus {
    paused: boolean
}

const onlineTasks = new Map<string, OnlineStatus>()

export function setTaskOnline(id: string) {
    onlineTasks.set(id, { paused: false })
}

export function isTaskOnline(id: string) {
    return onlineTasks.has(id)
}

export function clearTaskOnline(id: string) {
    onlineTasks.delete(id)
}

export function getTaskOnlineStatus(id: string) {
    return { ...onlineTasks.get(id), online: true }
}

export function toggleTaskPaused(id: string) {
    const s = onlineTasks.get(id)
    if (!s) return { paused: true, online: true }
    const after = { paused: !s.paused, online: true }
    onlineTasks.set(id, after)
    return after
}
