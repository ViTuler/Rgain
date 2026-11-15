<script setup lang="ts">
defineProps<{
  label: string;
  pressed?: boolean;
}>();
</script>

<template>
  <button
    type="button"
    class="icon_btn"
    :class="{ pressed }"
    :aria-label="label"
    :aria-pressed="pressed === undefined ? undefined : pressed"
  >
    <slot />
  </button>
</template>

<style scoped>
.icon_btn {
  display: inline-flex;
  width: 32px;
  height: 32px;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 0;
  border-radius: var(--radius_sm);
  background: transparent;
  color: var(--color_ink);
  transition: color 0.2s var(--ease_soft), background 0.2s var(--ease_soft);
}

.icon_btn:hover {
  color: var(--color_accent_deep);
  background: color-mix(in srgb, var(--color_accent) 12%, transparent);
}

.icon_btn.pressed {
  color: var(--color_accent_ink);
  background: color-mix(in srgb, var(--color_accent) 28%, transparent);
}

.icon_btn.pressed:hover {
  color: var(--color_accent_ink);
  background: color-mix(in srgb, var(--color_accent) 34%, transparent);
}

:global(html[data-theme="dark"]) .icon_btn.pressed {
  color: var(--color_on_accent);
  background: color-mix(in srgb, var(--color_accent) 32%, transparent);
}

.icon_btn :deep(svg) {
  width: 18px;
  height: 18px;
  display: block;
}
</style>
