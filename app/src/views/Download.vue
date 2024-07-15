<script lang="ts" setup>
// About
</script>

<template>
    <div class="w-full h-full flex justify-center items-center">
        <div class="flex flex-col w-full max-w-[1024px] h-full p-8 gap-2">
            <div class="flex flex-col gap-4">
                <div class="text-xl font-bold">{{ $t("download.subtitle") }}</div>
            </div>
            <div class="flex-none pt-4">
                <a class="btn btn-primary btn-lg btn-block" href="https://github.com/pa001024/weys/releases/latest" target="_blank"> {{ $t("download.github") }} </a>
            </div>
        </div>
    </div>
</template>
