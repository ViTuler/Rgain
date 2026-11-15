<script setup lang="ts">
import { computed } from "vue";
import type { GraphPayload } from "../../types/graph";

const props = defineProps<{
  sidecar_status: string;
  graph: GraphPayload | null;
}>();

const title = computed(() => props.graph?.meta.title || "3D 逻辑舞台");
const summary = computed(
  () =>
    props.graph?.meta.summary ||
    "黑白为底，青色画主干，红色点关键。色板对齐 Rain blog。",
);
const stats = computed(() => {
  if (!props.graph) return null;
  return {
    modules: props.graph.modules.length,
    nodes: props.graph.nodes.length,
    edges: props.graph.edges.length,
    axis: props.graph.meta.axis,
  };
});
</script>

<template>
  <section class="scene">
    <div class="scene_grid" />
    <div class="scene_copy">
      <p class="brand_mark">Rgain</p>
      <h1>{{ title }}</h1>
      <p class="desc">{{ summary }}</p>
      <div v-if="stats" class="graph_stats">
        <span>{{ stats.axis }}</span>
        <span>{{ stats.modules }} modules</span>
        <span>{{ stats.nodes }} nodes</span>
        <span>{{ stats.edges }} edges</span>
      </div>
      <div class="swatches" aria-hidden="true">
        <span class="swatch ink" title="ink" />
        <span class="swatch paper" title="paper" />
        <span class="swatch cyan" title="accent" />
        <span class="swatch red" title="brand" />
      </div>
      <p class="status">Sidecar：{{ sidecar_status }}</p>
      <p v-if="!graph" class="hint">Phase C 接 TresJS；当前显示解析摘要。</p>
    </div>
  </section>
</template>

<style scoped>
.scene {
  position: relative;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  background:
    radial-gradient(ellipse 70% 45% at 50% 0%, rgb(var(--accent_rgb) / 0.12), transparent),
    var(--color_night);
}

.scene_grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgb(var(--bright_rgb) / 0.06) 1px, transparent 1px),
    linear-gradient(90deg, rgb(var(--bright_rgb) / 0.06) 1px, transparent 1px);
  background-size: 48px 48px;
  mask-image: radial-gradient(ellipse at center, #000 35%, transparent 80%);
  opacity: 0.7;
  pointer-events: none;
}

.scene_copy {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--space_lg);
  text-align: center;
}

.brand_mark {
  margin: 0;
  font-size: 0.75rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--color_brand);
}

h1 {
  margin: 0.4rem 0 0.75rem;
  font-size: 1.75rem;
  font-weight: 650;
  color: var(--color_on_dark);
}

.desc {
  max-width: 32rem;
  margin: 0;
  color: var(--color_on_dark_soft);
}

.graph_stats {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  justify-content: center;
  margin-top: 0.85rem;
  font-size: 0.75rem;
  color: var(--color_accent);
}

.graph_stats span {
  padding: 0.2rem 0.55rem;
  border: 1px solid rgb(var(--bright_rgb) / 0.16);
  border-radius: 999px;
  background: rgb(var(--ink_rgb) / 0.35);
}

.swatches {
  display: flex;
  gap: 0.45rem;
  margin-top: var(--space_md);
}

.swatch {
  width: 1.1rem;
  height: 1.1rem;
  border-radius: 3px;
  border: 1px solid rgb(var(--bright_rgb) / 0.18);
}

.swatch.ink {
  background: #171414;
}

.swatch.paper {
  background: #f5efef;
}

.swatch.cyan {
  background: #75eec5;
}

.swatch.red {
  background: #ff1b1b;
}

.status {
  margin-top: var(--space_md);
  padding: 0.35rem 0.75rem;
  border-radius: 999px;
  border: 1px solid rgb(var(--bright_rgb) / 0.18);
  background: rgb(var(--ink_rgb) / 0.45);
  font-size: 0.8rem;
  color: var(--color_on_dark);
}

.hint {
  margin: 0.65rem 0 0;
  font-size: 0.72rem;
  color: var(--color_on_dark_soft);
}
</style>
