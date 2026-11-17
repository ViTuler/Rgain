<script setup lang="ts">
defineProps<{
    model_value: boolean;
    label: string;
}>();

const emit = defineEmits<{
    "update:model_value": [boolean];
}>();
</script>

<template>
    <label class="el_check">
        <input type="checkbox" :checked="model_value"
            @change="emit('update:model_value', ($event.target as HTMLInputElement).checked)" />
        <span class="el_box" aria-hidden="true" />
        <span class="el_label">{{ label }}</span>
    </label>
</template>

<style scoped>
.el_check {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    cursor: pointer;
    user-select: none;
    font-size: 13px;
    line-height: 1;
    color: var(--color_body);
}

.el_check input {
    position: absolute;
    width: 0;
    height: 0;
    margin: 0;
    opacity: 0;
}

.el_box {
    box-sizing: border-box;
    width: 14px;
    height: 14px;
    flex: none;
    border: 1px solid var(--color_line_strong);
    border-radius: 2px;
    background: var(--color_surface_3);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition:
        border-color 0.15s var(--ease_soft),
        background 0.15s var(--ease_soft);
}

.el_check:hover .el_box {
    border-color: var(--color_accent_deep);
}

.el_check input:focus-visible+.el_box {
    outline: 2px solid var(--color_accent_deep);
    outline-offset: 2px;
}

.el_check input:checked+.el_box {
    border-color: var(--color_accent_deep);
    background: var(--color_accent_deep);
}

.el_check input:checked+.el_box::after {
    content: "";
    width: 3px;
    height: 7px;
    margin-top: -1px;
    border: solid #fff;
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
}

.el_label {
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 12px;
    color: var(--color_ink);
}
</style>
