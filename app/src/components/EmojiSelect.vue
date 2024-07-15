<script lang="ts" setup>
import GraphemeSplitter from "grapheme-splitter"
import { ref } from "vue"
var splitter = new GraphemeSplitter()
function emoji(str: string | TemplateStringsArray) {
    if (typeof str === "string") {
        return [...splitter.splitGraphemes(str)]
    }
    return emoji(str.join(""))
}
const emojiList = {
    happy: emoji`😀😄😁😆😜🤣😂🤤🙂🙃😏🫠😉😊😇🥰😍🤩😘😗😚😙🥲😋😛😜🤪😝🤗🤭🥳😺😸😹😻😽`,
    sad: emoji`😟😔😕🙁😣😖😫😩🥱😢😭😿😵🥺😳😱💤`,
    angry: emoji`😠😡🤬😤🤡😈👿💢💥😾👻👺💀`,
    cofuse: emoji`🤔🙄🤤🤮😨😰😬😅😓😶😐😑😴😪💫💦`,
    body: emoji`👌👍👎🫰✌️🖕👋🤏✋👊🤛🤜💪👏🤝🫶🙏👂👀🧠🦷🦴`,
    animal: emoji`🐵🙈🙉🙊🐵🐒🦍🦧🐶🐕🦮🐕‍🦺🐩🐺🦊🦝🐱🐈🐈‍⬛🦁🐯🐅🐆🐴🫎🫏🐎🦄🦓🦌🦬🐮🐂🐃🐄🐷`,
    animal2: emoji`🐖🐗🐽🐏🐑🐐🐪🐫🦙🦒🐘🦣🦏🦛🐭🐁🐀🐹🐰🐇🐿️🦫🦔🦇🐻🐻‍❄️🐨🐼🦥🦦🦨🦘🦡🐾🦃🐔`,
    animal3: emoji`🐓🐣🐤🐥🐦🐧🕊️🦅🦆🦢🦉🦤🪶🦩🦚🦜🪽🐦‍⬛🪿🐦‍🔥🐸🐊🐢🦎🐍🐲🐉🦕🦖🐳🐋🐬🦭🐟🐠🐡`,
    animal4: emoji`🦈🐙🐚🪸🪼🐌🦋🐛🐜🐝🪲🐞🦗🪳🕷️🕸️🦂🦟🪰🪱🦠`,
    plant: emoji`💐🌸💮🪷🏵️🌹🥀🌺🌻🌼🌷🪻🌱🪴🌲🌳🌴🌵🌾🌿☘️🍀🍁🍂🍃🪹🪺🍄`,
    sign: emoji`💊❤💕💘💖💔⚠️🚫🔞❓✔️❌💩🏳️‍🌈🏳️🕶️👗📣🎵🎼🎤🎷📱☎️🪫💡📷💰🧬💉🚬🪦`,
    other: emoji`🌈🎉✨☀⭐☁️🌧️⚡💧🔥🌍⌛⏰🏆🍚🦀🦞🦐🥑🍆🥔🥕🌽🌶️🥜🧄🧅🥜🌰🍄‍🟫`,
    fruit: emoji`🍇🍈🍉🍊🍋🍋‍🟩🍌🍍🥭🍎🍏🍐🍑🍒🍓🫐🥝🍅🫒🥥`,
    sugar: emoji`🍦🍧🍨🍩🍪🎂🍰🧁🥧🍫🍬🍭🍮🍯`,
    drink: emoji`☕🍼🥛🫖🍵🍶🍾🍷🍸🍹🍺🍻🥂🥃🫗🥤🧋🧃🧉🧊`,
    tool: emoji`🍴🥢🍽️🥄🔪🔨🪓⛏️⚒️🛠️🗡️⚔️💣🪃🏹🛡️🪚🔧🪛🔩⚙️🗜️⚖️🦯🔗⛓️‍💥🪝🧰🧲🪜`,
    room: emoji`🚽🚪🛗🪞🪟🛏️🛋️🪑🚽🪠🚿🛁🪤🪒🧴🧷🧹🧺🧻🪣🧼🫧🪥🧽🧯🛒`,
} as Record<string, string[]>

const emit = defineEmits(["select"])
const tab = ref("happy")
const page = ref(0)
</script>

<template>
    <div class="flex flex-col gap-4">
        <div class="grid grid-cols-8 gap-2">
            <div class="btn btn-ghost btn-square btn-sm text-lg" v-for="item in emojiList[tab]" :key="item" @click="emit('select', item)">
                {{ item }}
            </div>
        </div>
        <div class="flex-none grid grid-cols-8 gap-2">
            <button v-if="page" class="btn btn-square btn-ghost btn-sm" @click="page = Math.max(0, page === 7 ? 0 : page - 6)">
                <Icon icon="la:prev" />
            </button>
            <button
                v-for="name in Object.keys(emojiList).slice(page, (page || 1) + 6)"
                :key="name"
                class="btn btn-square btn-ghost btn-sm"
                @click="tab = name"
                :class="tab === name ? 'bg-gray-200' : ''"
            >
                {{ emojiList[name][0] }}
            </button>
            <button v-if="Object.keys(emojiList).length > page + 6" class="btn btn-square btn-ghost btn-sm" @click="page = (page || 1) + 6">
                <Icon icon="la:next" />
            </button>
        </div>
    </div>
</template>
