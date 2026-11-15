<script setup lang="ts">
defineProps<{
  term: string;
  explanation: string;
  context?: string;
  busy?: boolean;
  error?: string;
  hotkey_hint?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <section class="sheet" role="dialog" aria-labelledby="term_title">
      <header>
        <div>
          <span class="eyebrow">划词解释</span>
          <h1 id="term_title">{{ term || "—" }}</h1>
        </div>
        <button type="button" class="btn_icon" aria-label="关闭" @click="emit('close')">
          ×
        </button>
      </header>

      <p v-if="context" class="context">上下文：{{ context }}</p>
      <p v-if="hotkey_hint" class="hint">快捷键 {{ hotkey_hint }} · 选中文本后触发；无选区时用当前节点</p>

      <div class="body">
        <p v-if="busy" class="muted">解释生成中…</p>
        <p v-else-if="error" class="error">{{ error }}</p>
        <p v-else-if="explanation">{{ explanation }}</p>
        <p v-else class="muted">选中文本后按快捷键，将在此显示释义并写入本地历史。</p>
      </div>
    </section>
  </div>
</template>

<style scoped>
.overlay {
  position: absolute;
  inset: 0;
  z-index: 30;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space_lg);
  background: rgb(var(--ink_rgb) / 0.55);
}

.sheet {
  width: min(420px, 100%);
  display: flex;
  flex-direction: column;
  gap: var(--space_sm);
  padding: var(--space_lg);
  border-radius: var(--radius_lg);
  border: 1px solid var(--color_line);
  background: var(--color_surface_3);
  box-shadow: var(--shadow_panel);
}

header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space_sm);
}

.eyebrow {
  display: block;
  font-size: 0.7rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color_brand);
}

h1 {
  margin: 0.2rem 0 0;
  font-size: 1.15rem;
  color: var(--color_ink);
}

.context {
  margin: 0;
  font-size: 0.78rem;
  color: var(--color_muted);
}

.hint {
  margin: 0;
  font-size: 0.72rem;
  color: var(--color_faint);
}

.body {
  min-height: 4.5rem;
  padding: var(--space_md);
  border-radius: var(--radius_md);
  background: var(--color_surface_2);
  border: 1px solid var(--color_line);
  color: var(--color_body);
  line-height: 1.55;
  white-space: pre-wrap;
}

.muted {
  margin: 0;
  color: var(--color_faint);
}

.error {
  margin: 0;
  color: var(--color_brand);
}

.btn_icon {
  width: 2rem;
  height: 2rem;
  padding: 0;
  border: 0;
  border-radius: var(--radius_sm);
  font-size: 1.2rem;
  line-height: 1;
  color: var(--color_muted);
  background: transparent;
}
</style>
