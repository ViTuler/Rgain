<script setup lang="ts">
defineProps<{
    message: string;
    tone?: "info" | "ok" | "error";
}>();

const emit = defineEmits<{
    close: [];
}>();
</script>

<template>
    <div class="status_toast" :class="`tone_${tone || 'info'}`" role="status" aria-live="polite">
        <p class="toast_text">{{ message }}</p>
        <button type="button" class="toast_close" aria-label="close" @click="emit('close')">
            ×
        </button>
    </div>
</template>

<style scoped>
.status_toast {
    position: absolute;
    left: 50%;
    bottom: 0.85rem;
    z-index: 45;
    display: flex;
    align-items: flex-start;
    gap: 0.55rem;
    max-width: min(42rem, calc(100% - 1.5rem));
    padding: 0.55rem 0.65rem 0.55rem 0.8rem;
    border: 1px solid var(--color_line_strong);
    border-radius: 8px;
    background: color-mix(in srgb, var(--color_surface_3) 92%, var(--color_surface));
    box-shadow: 0 10px 28px color-mix(in srgb, var(--color_night) 18%, transparent);
    transform: translateX(-50%);
    backdrop-filter: blur(8px);
}

.tone_ok {
    border-color: color-mix(in srgb, var(--color_accent) 45%, var(--color_line_strong));
}

.tone_error {
    border-color: color-mix(in srgb, var(--color_brand) 42%, var(--color_line_strong));
}

.toast_text {
    margin: 0;
    flex: 1;
    min-width: 0;
    font-size: 0.78rem;
    line-height: 1.45;
    color: var(--color_body);
    white-space: pre-wrap;
    word-break: break-word;
}

.toast_close {
    flex: 0 0 auto;
    width: 1.35rem;
    height: 1.35rem;
    margin: 0;
    padding: 0;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--color_muted);
    font-size: 1rem;
    line-height: 1;
    cursor: pointer;
}

.toast_close:hover {
    color: var(--color_ink);
    background: color-mix(in srgb, var(--color_ink) 6%, transparent);
}
</style>
