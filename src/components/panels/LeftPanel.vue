<script setup lang="ts">
defineProps<{
  project_label: string;
  runs_count: number;
}>();

const emit = defineEmits<{
  open_settings: [];
}>();
</script>

<template>
  <aside class="left_panel">
    <header class="panel_head">
      <span class="eyebrow">Project</span>
      <strong>{{ project_label }}</strong>
    </header>

    <section class="block">
      <h2>目录</h2>
      <p class="muted">Phase B 接入本地目录扫描。当前为占位。</p>
      <button type="button" class="btn_ghost" disabled>选择目录…</button>
    </section>

    <section class="block">
      <h2>历史 Runs</h2>
      <p class="muted">已缓存 {{ runs_count }} 条（骨架）。</p>
      <ul class="empty_list">
        <li>暂无解析记录</li>
      </ul>
    </section>

    <div class="spacer" />

    <button type="button" class="btn_mint" @click="emit('open_settings')">
      设置
    </button>
  </aside>
</template>

<style scoped>
.left_panel {
  display: flex;
  flex-direction: column;
  gap: var(--space_md);
  width: var(--panel_left_width);
  min-width: var(--panel_left_width);
  padding: var(--space_md);
  border-right: 1px solid var(--color_border);
  background: var(--color_bg_panel);
}

.panel_head {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.eyebrow {
  font-size: 0.7rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color_accent_mint);
}

.block h2 {
  margin: 0 0 var(--space_xs);
  font-size: 0.85rem;
  font-weight: 600;
}

.muted {
  margin: 0 0 var(--space_sm);
  color: var(--color_text_muted);
  font-size: 0.8rem;
}

.empty_list {
  margin: 0;
  padding-left: 1.1rem;
  color: var(--color_text_muted);
  font-size: 0.8rem;
}

.spacer {
  flex: 1;
}

.btn_mint,
.btn_ghost {
  border-radius: var(--radius_sm);
  border: 1px solid transparent;
  padding: 0.55rem 0.85rem;
}

.btn_mint {
  color: var(--color_text_on_mint);
  background: var(--color_accent_mint);
}

.btn_mint:hover {
  filter: brightness(1.05);
}

.btn_ghost {
  width: 100%;
  color: var(--color_text_muted);
  background: transparent;
  border-color: var(--color_border);
}

.btn_ghost:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
</style>
