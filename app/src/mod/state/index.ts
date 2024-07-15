import { computed } from "vue"

export function useState<T, N extends keyof T>(obj: T, key: N) {
    return [computed(() => obj[key]), (val: T[N]) => (obj[key] = val)] as const
}

import type { Ref } from "vue"
import type { Observable } from "dexie"
declare module "@vueuse/rxjs" {
    export function useObservable<H, I = undefined>(observable: Observable<H>): Readonly<Ref<H | I>>
}
