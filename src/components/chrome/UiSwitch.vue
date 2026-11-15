<script setup lang="ts">
defineProps<{
    model_value: boolean;
    disabled?: boolean;
    label?: string;
}>();

const emit = defineEmits<{
    "update:model_value": [boolean];
}>();

function toggle(disabled: boolean | undefined, current: boolean) {
    if (disabled) return;
    emit("update:model_value", !current);
}
</script>

<template>
    <button
        type="button"
        class="ui_switch"
        role="switch"
        :aria-checked="model_value"
        :aria-label="label"
        :disabled="disabled"
        :data-on="model_value ? '1' : '0'"
        @click="toggle(disabled, model_value)"
    >
        <span class="track" aria-hidden="true">
            <span class="thumb" />
        </span>
    </button>
</template>

<style scoped>
.ui_switch {
    display: inline-flex;
    align-items: center;
    padding: 0;
    border: 0;
    background: transparent;
    cursor: pointer;
}

.ui_switch:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.track {
    position: relative;
    width: 40px;
    height: 22px;
    border-radius: 999px;
    background: var(--color_line_strong);
    transition: background 0.2s var(--ease_soft);
}

.ui_switch[data-on="1"] .track {
    background: var(--color_accent_deep);
}

.thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--color_surface_3);
    box-shadow: 0 1px 3px rgb(var(--ink_rgb) / 0.22);
    transition: transform 0.2s var(--ease_soft);
}

.ui_switch[data-on="1"] .thumb {
    transform: translateX(18px);
}
</style>
