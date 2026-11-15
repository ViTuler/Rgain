<script setup lang="ts">
import { use_i18n } from "../../i18n";

defineProps<{
    visible: boolean;
    left: number;
    top: number;
}>();

const emit = defineEmits<{
    explain: [];
}>();

const { t } = use_i18n();
</script>

<template>
    <Transition name="fab_fade">
        <button v-if="visible" type="button" class="selection_fab" :style="{ left: `${left}px`, top: `${top}px` }"
            :title="t('term.fab_title')" :aria-label="t('term.fab_title')" @mousedown.prevent @click="emit('explain')">
            <svg viewBox="0 0 24 24" aria-hidden="true">
                <path fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"
                    d="M10.5 4.5a6 6 0 1 1 0 12 6 6 0 0 1 0-12zM15.8 15.8 20 20" />
            </svg>
        </button>
    </Transition>
</template>

<style scoped>
.selection_fab {
    position: fixed;
    z-index: 18;
    display: inline-flex;
    width: 20px;
    height: 20px;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 1px solid var(--color_line);
    border-radius: 999px;
    background: color-mix(in srgb, var(--color_surface_3) 94%, transparent);
    color: var(--color_ink);
    box-shadow: var(--shadow_panel);
    cursor: pointer;
    transform: translate(4px, calc(-100% - 4px));
    backdrop-filter: blur(8px);
}

.selection_fab:hover {
    border-color: color-mix(in srgb, var(--color_accent) 45%, var(--color_line));
    color: var(--color_accent_deep);
}

.selection_fab svg {
    width: 12px;
    height: 12px;
}

.fab_fade-enter-active,
.fab_fade-leave-active {
    transition:
        opacity 0.14s var(--ease_soft),
        transform 0.14s var(--ease_soft);
}

.fab_fade-enter-from,
.fab_fade-leave-to {
    opacity: 0;
    transform: translate(4px, calc(-100% - 10px));
}
</style>
