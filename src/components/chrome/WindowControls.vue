<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted, onUnmounted, ref } from "vue";

const is_maximized = ref(false);
const app_window = getCurrentWindow();
let unlisten: (() => void) | undefined;

async function refresh_maximized() {
    try {
        is_maximized.value = await app_window.isMaximized();
    } catch {
        is_maximized.value = false;
    }
}

async function on_minimize() {
    await app_window.minimize();
}

async function on_toggle_maximize() {
    await app_window.toggleMaximize();
    await refresh_maximized();
}

async function on_close() {
    await app_window.close();
}

onMounted(async () => {
    await refresh_maximized();
    try {
        unlisten = await app_window.onResized(() => {
            void refresh_maximized();
        });
    } catch {
        // browser preview without tauri
    }
});

onUnmounted(() => {
    unlisten?.();
});
</script>

<template>
    <div class="window_controls" aria-label="窗口控制">
        <button type="button" class="traffic yellow" aria-label="最小化" title="最小化" @click="on_minimize">
            <span class="glyph minus" />
        </button>
        <button type="button" class="traffic green" :aria-label="is_maximized ? '还原' : '最大化'"
            :title="is_maximized ? '还原' : '最大化'" @click="on_toggle_maximize">
            <span class="glyph" :class="is_maximized ? 'restore' : 'plus'" />
        </button>
        <button type="button" class="traffic red" aria-label="关闭" title="关闭" @click="on_close">
            <span class="glyph close" />
        </button>
    </div>
</template>

<style scoped>
.window_controls {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-left: 0.35rem;
    /* Keep traffic lights out of the drag region hit-test */
}

.traffic {
    position: relative;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    cursor: pointer;
    transition: filter 0.15s var(--ease_soft), transform 0.12s var(--ease_soft);
}

.traffic:hover {
    filter: brightness(1.06);
}

.traffic:active {
    transform: scale(0.92);
}

.traffic.yellow {
    background: #febc2e;
}

.traffic.green {
    background: #28c840;
}

.traffic.red {
    background: #ff5f57;
}

.glyph {
    position: absolute;
    inset: 0;
    margin: auto;
    opacity: 0;
    transition: opacity 0.12s var(--ease_soft);
    pointer-events: none;
}

.window_controls:hover .glyph {
    opacity: 0.72;
}

.glyph.minus {
    width: 6px;
    height: 1px;
    background: #7a5200;
}

.glyph.plus {
    width: 6px;
    height: 6px;
}

.glyph.plus::before,
.glyph.plus::after {
    content: "";
    position: absolute;
    left: 50%;
    top: 50%;
    background: #0b5b1a;
    transform: translate(-50%, -50%);
}

.glyph.plus::before {
    width: 6px;
    height: 1px;
}

.glyph.plus::after {
    width: 1px;
    height: 6px;
}

.glyph.restore {
    width: 5px;
    height: 5px;
    border: 1px solid #0b5b1a;
    box-sizing: border-box;
}

.glyph.close {
    width: 6px;
    height: 6px;
}

.glyph.close::before,
.glyph.close::after {
    content: "";
    position: absolute;
    left: 50%;
    top: 50%;
    width: 7px;
    height: 1px;
    background: #6b0f0a;
}

.glyph.close::before {
    transform: translate(-50%, -50%) rotate(45deg);
}

.glyph.close::after {
    transform: translate(-50%, -50%) rotate(-45deg);
}
</style>
