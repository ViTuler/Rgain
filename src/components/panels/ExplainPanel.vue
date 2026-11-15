<script setup lang="ts">
import type { GraphNode } from "../../types/graph";

defineProps<{
  open: boolean;
  selected_label: string;
  selected_node: GraphNode | null;
  graph_title: string;
}>();
</script>

<template>
  <aside
    class="explain_drawer"
    :class="{ open }"
    :aria-hidden="!open"
    aria-labelledby="explain_title"
  >
    <header class="panel_head">
      <span class="eyebrow">Explain</span>
      <strong id="explain_title">{{ selected_label || "未选中节点" }}</strong>
      <span v-if="graph_title" class="graph_title">{{ graph_title }}</span>
    </header>

    <section class="block">
      <h2>讲解</h2>
      <template v-if="selected_node">
        <p class="meta_row">
          <span class="chip" :data-kind="selected_node.kind">{{ selected_node.kind }}</span>
          <span class="chip">{{ selected_node.module_id }}</span>
          <span class="chip">order {{ selected_node.axis_order }}</span>
        </p>
        <p class="body">{{ selected_node.explain }}</p>
        <div v-if="selected_node.source_refs?.length" class="refs">
          <h3>源引用</h3>
          <ul>
            <li v-for="(ref, i) in selected_node.source_refs" :key="`${ref.path}-${i}`">
              <code>{{ ref.path }}</code>
              <span v-if="ref.span" class="span">{{ ref.span }}</span>
            </li>
          </ul>
        </div>
      </template>
      <template v-else>
        <p class="muted">在 2D / 3D 舞台点击节点后，此处展示 explain 与源引用。</p>
        <div class="empty_box">空态 · 等待选中</div>
      </template>
    </section>
  </aside>
</template>

<style scoped>
.explain_drawer {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  z-index: 15;
  display: flex;
  flex-direction: column;
  gap: var(--space_md);
  width: min(var(--panel_right_width), 92vw);
  padding: var(--space_md);
  border-left: 1px solid var(--color_line);
  background: var(--color_surface);
  box-shadow: -12px 0 36px -18px rgb(var(--ink_rgb) / 0.35);
  transform: translateX(100%);
  transition: transform 0.24s var(--ease_soft);
  pointer-events: none;
}

.explain_drawer.open {
  transform: translateX(0);
  pointer-events: auto;
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
  color: var(--color_accent_deep);
}

.graph_title {
  font-size: 0.72rem;
  color: var(--color_faint);
}

.block h2 {
  margin: 0 0 var(--space_xs);
  font-size: 0.85rem;
  color: var(--color_ink);
}

.block h3 {
  margin: 0.75rem 0 0.35rem;
  font-size: 0.78rem;
  color: var(--color_muted);
}

.muted {
  margin: 0 0 var(--space_sm);
  color: var(--color_muted);
  font-size: 0.8rem;
}

.body {
  margin: 0.5rem 0 0;
  color: var(--color_body);
  font-size: 0.88rem;
  line-height: 1.55;
  white-space: pre-wrap;
}

.meta_row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin: 0;
}

.chip {
  padding: 0.12rem 0.45rem;
  border-radius: 999px;
  border: 1px solid var(--color_line);
  font-size: 0.68rem;
  color: var(--color_muted);
  background: var(--color_surface_2);
}

.chip[data-kind="trunk"] {
  border-color: color-mix(in srgb, var(--color_accent) 45%, var(--color_line));
  color: var(--color_accent_deep);
}

.chip[data-kind="branch"] {
  border-color: color-mix(in srgb, var(--color_brand) 45%, var(--color_line));
  color: var(--color_brand);
}

.refs ul {
  margin: 0;
  padding-left: 1rem;
  font-size: 0.75rem;
  color: var(--color_body);
}

.refs code {
  font-size: 0.72rem;
  word-break: break-all;
}

.span {
  margin-left: 0.35rem;
  color: var(--color_faint);
}

.empty_box {
  border: 1px dashed var(--color_line_strong);
  border-radius: var(--radius_md);
  padding: var(--space_md);
  color: var(--color_faint);
  font-size: 0.8rem;
  text-align: center;
  background: var(--color_surface_2);
}
</style>
