<script setup lang="ts">
defineProps<{
  theme: "light" | "dark";
}>();

const emit = defineEmits<{
  toggle: [];
}>();
</script>

<template>
  <button
    type="button"
    class="icon_btn theme_switch"
    :class="{ is_dark: theme === 'dark' }"
    :aria-label="theme === 'dark' ? '切换到浅色主题' : '切换到深色主题'"
    :aria-pressed="theme === 'dark'"
    @click="emit('toggle')"
  >
    <svg class="icon sun" viewBox="0 0 24 24" aria-hidden="true">
      <circle cx="12" cy="12" r="3.5" fill="none" stroke="currentColor" stroke-width="1.5" />
      <path
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        d="M12 3.5v2.2M12 18.3v2.2M3.5 12h2.2M18.3 12h2.2M6 6l1.6 1.6M16.4 16.4 18 18M18 6l-1.6 1.6M7.6 16.4 6 18"
      />
    </svg>
    <svg class="icon moon" viewBox="0 0 24 24" aria-hidden="true">
      <path
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linejoin="round"
        d="M15.5 3.5a7.5 7.5 0 1 0 5 12.5A6 6 0 0 1 15.5 3.5z"
      />
    </svg>
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

.icon {
  width: 18px;
  height: 18px;
}

.sun {
  display: block;
}

.moon {
  display: none;
}

.theme_switch.is_dark .sun {
  display: none;
}

.theme_switch.is_dark .moon {
  display: block;
}
</style>
