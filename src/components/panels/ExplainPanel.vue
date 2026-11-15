<script setup lang="ts">
defineProps<{
  selected_label: string;
}>();
</script>

<template>
  <aside class="explain_panel">
    <header class="panel_head">
      <span class="eyebrow">Explain</span>
      <strong>{{ selected_label || "未选中节点" }}</strong>
    </header>

    <section class="block">
      <h2>讲解</h2>
      <p class="muted">
        点击 3D 节点后，此处展示模型返回的 explain 文案与源引用。
      </p>
      <div class="empty_box">空态 · 等待图数据</div>
    </section>

    <section class="block">
      <h2>划词解释</h2>
      <p class="muted">快捷键在设置中配置（Phase D 接通 API）。</p>
      <div class="hint_row">
        <span class="dot crimson" />
        选中文本后触发解释浮层
      </div>
    </section>
  </aside>
</template>

<style scoped>
.explain_panel {
  display: flex;
  flex-direction: column;
  gap: var(--space_md);
  width: var(--panel_right_width);
  min-width: var(--panel_right_width);
  padding: var(--space_md);
  border-left: 1px solid var(--color_border);
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
  color: var(--color_accent_crimson);
}

.block h2 {
  margin: 0 0 var(--space_xs);
  font-size: 0.85rem;
}

.muted {
  margin: 0 0 var(--space_sm);
  color: var(--color_text_muted);
  font-size: 0.8rem;
}

.empty_box {
  border: 1px dashed var(--color_border);
  border-radius: var(--radius_md);
  padding: var(--space_md);
  color: var(--color_text_muted);
  font-size: 0.8rem;
  text-align: center;
}

.hint_row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
  color: var(--color_text_on_deep);
}

.dot {
  width: 0.55rem;
  height: 0.55rem;
  border-radius: 50%;
  background: var(--color_accent_mint);
}

.dot.crimson {
  background: var(--color_accent_crimson);
}
</style>
