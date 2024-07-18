<script lang="ts" setup>
import * as dialog from "@tauri-apps/plugin-dialog"
import { useGameStore } from "../mod/state/game"
import { useTranslation } from "i18next-vue"
import { CollapsibleContent, CollapsibleRoot, CollapsibleTrigger } from "radix-vue"
import gsap from "gsap"
import { FunctionDirective, ref, watchEffect, nextTick, onMounted } from "vue"

const game = useGameStore()
const { t } = useTranslation()

const keys = ["path", "beforeGame", "afterGame"] as const

async function selectPath(key: "path" | "beforeGame" | "afterGame") {
    const result = await dialog.open({
        defaultPath: game[key],
        filters:
            key === "path"
                ? [{ name: "YuanShen.exe", extensions: ["exe"] }]
                : [
                      { name: t("misc.exec_files"), extensions: ["exe", "bat", "cmd", "ahk", "ps1"] },
                      { name: t("misc.all_files"), extensions: ["*"] },
                  ],
    })
    if (result && result.path) {
        game[key] = result.path
    }
}

async function clearAccounts() {
    if (await dialog.confirm(t("game.clearAccountsConfirm"))) {
        game.clearAccounts()
    }
}

function spin_y_enter(el: any, done: () => void) {
    gsap.from(el, { duration: 0.3, rotateX: 180, onComplete: done })
}
function spin_y_leave(el: any, done: () => void) {
    gsap.set(el, { duration: 0.3, position: "absolute", onComplete: done })
}

const scrollRef = ref<HTMLDivElement | null>(null)

function scrollToCenter(id: number = game.selected) {
    const viewNode = scrollRef.value?.children[0]
    if (!viewNode) return
    viewNode.children[game.accounts!.findIndex((acc) => acc.id === id)].scrollIntoView({ behavior: "smooth", block: "center" })
}

async function switchAccount(id: number) {
    game.selected = id
    await game.switchAccount(id)
}

async function importAccounts() {
    const select_json = () =>
        new Promise((resolve) => {
            const input = document.createElement("input")
            input.type = "file"
            input.accept = ".json"
            input.onchange = () => {
                const file = input.files?.item(0)
                if (file) {
                    const reader = new FileReader()
                    reader.onload = (e) => {
                        const json = e.target?.result as string
                        if (json) {
                            resolve(JSON.parse(json))
                        }
                    }
                    reader.readAsText(file)
                }
            }
            input.click()
        })
    const json = await select_json()
    if (json) {
        game.import_accounts(json)
    }
}

function exportAccounts() {
    const data = game.export_accounts()
    const blob = new Blob([data], { type: "text/json;charset=utf-8" })

    const a = document.createElement("a")
    a.href = URL.createObjectURL(blob)
    a.download = "export_accounts.json"
    a.click()
}

const editId = ref(0)
const editValue = ref("")

watchEffect(() => {
    if (game.selected) {
        scrollToCenter(game.selected)
    }
})

function selectPrev(e: PointerEvent) {
    if (game.selectPrev() && e.button === 2) {
        game.launchGame()
    }
}
function selectNext(e: PointerEvent) {
    if (game.selectNext() && e.button === 2) {
        game.launchGame()
    }
}

const vFocus: FunctionDirective = (el) => {
    setTimeout(() => {
        el.focus()
    }, 1)
    el.onblur = () => {
        if (game.selected === editId.value) {
            editId.value = 0
            game.updateName(editValue.value)
        }
        el.onblur = null
    }
}
onMounted(async () => {
    await nextTick()
    await game.checkCurrentAccount()
    await game.addAccountReg()
})
function copyUID(uid?: string) {
    if (uid) navigator.clipboard.writeText(uid)
}
</script>

<template>
    <div class="flex flex-col w-full h-full overflow-hidden p-4 space-y-2">
        <CollapsibleRoot class="space-y-2" v-model:open="game.expend">
            <div class="join w-full">
                <div v-if="game.running" @click="game.launchGame" class="join-item btn btn-primary btn-disabled flex-1">
                    {{ $t("game.launched") }}
                </div>
                <div v-else @click="game.launchGame" class="join-item btn btn-primary flex-1">{{ $t("game.launch") }}</div>
                <CollapsibleTrigger class="join-item btn btn-primary inline-flex items-center justify-center">
                    <Transition @enter="spin_y_enter" @leave="spin_y_leave" :css="false">
                        <Icon icon="radix-icons:chevron-up" v-if="game.expend" class="size-5" />
                        <Icon icon="radix-icons:chevron-down" v-else class="size-5" />
                    </Transition>
                </CollapsibleTrigger>
            </div>

            <CollapsibleContent class="data-[state=open]:animate-slideDown data-[state=closed]:animate-slideUp overflow-hidden">
                <div v-for="key in keys" :key="key">
                    <div class="form-control flex flex-row justify-between items-center flex-wrap">
                        <label class="label cursor-pointer space-x-2 min-w-32 justify-start">
                            <input type="checkbox" v-model="game[`${key}Enable`]" class="checkbox checkbox-primary" />
                            <span class="label-text">{{ $t("game." + key) }}</span>
                        </label>
                        <div class="flex flex-1 space-x-2" v-show="game[`${key}Enable`]">
                            <input
                                type="text"
                                disabled
                                :value="game[key]"
                                :placeholder="$t('misc.selectPath')"
                                class="input input-bordered input-sm w-full min-w-32"
                            />
                            <div class="btn btn-primary btn-sm" @click="selectPath(key)">{{ $t("misc.select") }}</div>
                        </div>
                    </div>
                    <div
                        class="form-control flex flex-row justify-between items-center flex-wrap"
                        v-if="key === 'path' && game[`${key}Enable`]"
                    >
                        <label class="label cursor-pointer min-w-32 justify-start">
                            <span class="label-text ml-12">{{ $t("game.params") }}</span>
                        </label>
                        <div class="flex flex-1 space-x-2" v-show="game.pathEnable">
                            <input
                                v-if="!game.autoLoginEnable"
                                type="text"
                                v-model="game.pathParams"
                                class="input input-bordered input-sm w-full min-w-32"
                            />
                            <input
                                v-else
                                type="text"
                                disabled
                                value="-screen-width 1600 -screen-height 900 -platform_type CLOUD_THIRD_PARTY_MOBILE"
                                class="input input-bordered input-sm w-full min-w-32"
                            />
                        </div>
                    </div>
                </div>
                <div>
                    <div class="form-control flex flex-row justify-between items-center flex-wrap">
                        <label class="label cursor-pointer space-x-2 min-w-32 justify-start">
                            <input type="checkbox" v-model="game.autoLoginEnable" class="checkbox checkbox-primary" />
                            <span class="label-text">{{ $t("game.autoLogin") }}</span>
                        </label>
                        <div v-if="game.autoLoginEnable" class="flex flex-1 items-center space-x-2">
                            <span class="text-sm">{{ $t("game.autoSend") }}</span>
                            <Select v-model="game.autoLoginRoom">
                                <SelectItem value="-">{{ $t("game.autoLoginRoom") }}</SelectItem>
                                <GQQuery query="query { rooms { id, name } }" :variables="{}" v-slot="{ data }">
                                    <SelectItem v-if="data" v-for="item in data.rooms" :key="item.id" :value="item.id">{{
                                        item.name
                                    }}</SelectItem>
                                </GQQuery>
                            </Select>
                        </div>
                    </div>
                </div>
            </CollapsibleContent>
        </CollapsibleRoot>

        <div class="mb-4 space-x-2 flex">
            <Tooltip :tooltip="$t('game.addAccountReg')" side="top">
                <CheckAnimationButton icon="la:plus-solid" @click="game.addAccountReg" @contextmenu.prevent="game.addNewAccountReg" />
            </Tooltip>
            <Tooltip :tooltip="$t('game.importAccountsTooltip')" side="top">
                <CheckAnimationButton
                    icon="la:paste-solid"
                    @click="game.importAccountsFromCliboard"
                    :aria-label="$t('game.importAccounts')"
                />
            </Tooltip>
            <Tooltip :tooltip="$t('game.clearAccounts')" side="top">
                <CheckAnimationButton icon="la:broom-solid" @click="clearAccounts" />
            </Tooltip>
            <Tooltip :tooltip="$t('game.import')" side="top">
                <CheckAnimationButton noanimate icon="la:import" @click="importAccounts" />
            </Tooltip>
            <Tooltip :tooltip="$t('game.export')" side="top">
                <CheckAnimationButton noanimate icon="la:export" @click="exportAccounts" />
            </Tooltip>
            <div class="flex-1"></div>
            <Tooltip :tooltip="$t('game.prev')" side="top">
                <CheckAnimationButton icon="la:prev" @click="selectPrev" @contextmenu.prevent="selectPrev" />
            </Tooltip>
            <Tooltip :tooltip="$t('game.next')" side="top">
                <CheckAnimationButton icon="la:next" @click="selectNext" @contextmenu.prevent="selectNext" />
            </Tooltip>
        </div>
        <div class="bg-base-100 p-4 w-full justify-items-center rounded-lg flex-1 flex flex-col overflow-hidden">
            <ScrollArea class="overflow-hidden flex-1" @loadref="(r) => (scrollRef = r)">
                <div class="label cursor-pointer space-x-2 group h-9" v-for="(acc, index) in game.accounts" :key="acc.id">
                    <label class="label space-x-2 p-0">
                        <span class="label-text min-w-6">{{ index + 1 }}.</span>
                        <input
                            type="radio"
                            name="radio-10"
                            class="radio radio-secondary radio-sm"
                            :value="acc.id"
                            v-model="game.selected"
                        />
                        <Tooltip v-if="acc.token" :tooltip="$t('game.accountTypeU')" side="top">
                            <div
                                class="rounded text-sm border p-0.5 px-1.5 size-6 text-center whitespace-nowrap border-secondary text-secondary"
                            >
                                R
                            </div>
                        </Tooltip>
                        <Tooltip v-if="acc.login && acc.pwd" :tooltip="$t('game.accountTypeP')" side="top">
                            <div class="rounded text-sm border p-0.5 px-1.5 size-6 text-center whitespace-nowrap">P</div>
                        </Tooltip>
                        <template v-if="acc.id !== editId">
                            <span class="label-text font-bold" v-if="acc.name">{{ acc.name }}</span>
                            <span
                                class="label-text flex-1 text-ellipsis overflow-hidden whitespace-nowrap"
                                @contextmenu.prevent="copyUID(acc.uid)"
                            >
                                {{
                                    (acc.uid && `UID:${acc.uid}${acc.login ? ` (${acc.login})` : ""}`) || acc.login || acc.hash.slice(0, 12)
                                }}
                            </span>
                            <button
                                class="btn btn-sm btn-square group-hover:opacity-100 opacity-0"
                                @click=";(editId = acc.id), (editValue = acc.name)"
                            >
                                <Icon icon="mdl2:rename" />
                            </button>
                        </template>
                        <div v-else>
                            <input class="input input-xs w-full min-w-32" type="text" v-model="editValue" v-focus />
                        </div>
                    </label>
                    <div class="join group-hover:opacity-100 opacity-0 transition-all duration-300 pr-4">
                        <Tooltip v-if="acc.login && acc.pwd" :tooltip="$t('game.copyTooltip')" side="top">
                            <div class="btn btn-sm join-item" @click.stop="game.copyAccount(acc.id)">
                                <Icon icon="la:copy-solid" />
                            </div>
                        </Tooltip>
                        <Tooltip :tooltip="$t('game.deleteTooltip')" side="top">
                            <div class="btn btn-sm join-item" @click.stop="game.deleteAccount(acc.id)" @contextmenu.prevent="game.deleteReg(acc.id)">
                                <Icon icon="la:trash-alt" />
                            </div>
                        </Tooltip>
                        <Tooltip :tooltip="$t('game.lockTooltip')" side="top">
                            <div class="btn btn-sm join-item" @click.stop="game.lockAccount(acc.id)">
                                <Icon v-if="acc.lock" icon="la:lock-solid" />
                                <Icon v-else icon="la:lock-open-solid" />
                            </div>
                        </Tooltip>
                        <Tooltip :tooltip="$t('game.switchTooltip')" side="top">
                            <div class="btn btn-sm join-item" @click.stop="switchAccount(acc.id)">
                                <Icon icon="la:exchange-alt-solid" />
                            </div>
                        </Tooltip>
                    </div>
                </div>
            </ScrollArea>
        </div>
    </div>
</template>
