<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { TresCanvas } from "@tresjs/core";
import { CameraControls } from "@tresjs/cientos";
import type { GraphPayload } from "../../types/graph";
import {
  demo_graph,
  layout_trunk_graph,
  type LaidOutNode,
} from "../../lib/layout_trunk";
import GraphNodeMesh from "./GraphNodeMesh.vue";
import GraphEdgeLine from "./GraphEdgeLine.vue";

const props = defineProps<{
  graph: GraphPayload | null;
  sidecar_status: string;
}>();

const emit = defineEmits<{
  select_node: [node: LaidOutNode];
}>();

const selected_id = ref<string | null>(null);
const hovered_id = ref<string | null>(null);
/** Radians: wheel spins the graph around the main X axis (YZ branches rotate). */
const axis_spin = ref(0);

const scene_host = ref<HTMLElement | null>(null);

const source_graph = computed(() => props.graph ?? demo_graph());
const is_demo = computed(() => !props.graph);
const layout = computed(() => layout_trunk_graph(source_graph.value));

const neighbor_ids = computed(() => {
  const focus = hovered_id.value || selected_id.value;
  if (!focus) return new Set<string>();
  const set = new Set<string>([focus]);
  for (const e of layout.value.edges) {
    if (e.from === focus) set.add(e.to);
    if (e.to === focus) set.add(e.from);
  }
  return set;
});

const active_edge_keys = computed(() => {
  const focus = hovered_id.value || selected_id.value;
  if (!focus) return new Set<string>();
  const set = new Set<string>();
  for (const e of layout.value.edges) {
    if (e.from === focus || e.to === focus) {
      set.add(`${e.from}->${e.to}`);
    }
  }
  return set;
});

const has_focus = computed(() => Boolean(hovered_id.value || selected_id.value));

const camera_position = computed(() => {
  const c = layout.value.bounds.center;
  const span_x = Math.max(
    6,
    layout.value.bounds.max.x - layout.value.bounds.min.x,
  );
  // Look at the X spine from +Z, slightly above.
  return [c.x, c.y + 1.8, c.z + Math.max(10, span_x * 0.85)] as [
    number,
    number,
    number,
  ];
});

const controls_target = computed(() => {
  const c = layout.value.bounds.center;
  return [c.x, c.y, c.z] as [number, number, number];
});

watch(
  () => props.graph,
  () => {
    selected_id.value = null;
    hovered_id.value = null;
    axis_spin.value = 0;
  },
);

function on_select(id: string) {
  selected_id.value = id;
  const node = layout.value.nodes.find((n) => n.id === id);
  if (node) emit("select_node", node);
}

function on_hover(id: string | null) {
  hovered_id.value = id;
}

function on_wheel(ev: WheelEvent) {
  ev.preventDefault();
  ev.stopPropagation();
  // Positive deltaY → rotate so YZ branches spin around X.
  axis_spin.value += Math.sign(ev.deltaY) * 0.12 + ev.deltaY * 0.0015;
}

onMounted(() => {
  const el = scene_host.value;
  if (!el) return;
  el.addEventListener("wheel", on_wheel, { passive: false, capture: true });
});

onBeforeUnmount(() => {
  scene_host.value?.removeEventListener("wheel", on_wheel, true);
});
</script>

<template>
  <section class="scene">
    <div ref="scene_host" class="canvas_host">
      <TresCanvas clear-color="#100e0e">
        <TresPerspectiveCamera :position="camera_position" />
        <!-- Drag to orbit/pan; wheel is reserved for X-axis spin (handled on host). -->
        <CameraControls
          make-default
          :target="controls_target"
          :dolly-speed="0"
        />
        <TresAmbientLight :intensity="0.55" />
        <TresDirectionalLight :position="[8, 6, 10]" :intensity="1.05" />
        <TresDirectionalLight
          :position="[-4, 2, -6]"
          :intensity="0.35"
          color="#75eec5"
        />

        <!-- Spin around X: trunk stays on axis, YZ branches rotate -->
        <TresGroup
          :position="[controls_target[0], controls_target[1], controls_target[2]]"
        >
          <TresGroup :rotation="[axis_spin, 0, 0]">
            <TresGroup
              :position="[-controls_target[0], -controls_target[1], -controls_target[2]]"
            >
              <GraphEdgeLine
                v-for="edge in layout.edges"
                :key="`${edge.from}->${edge.to}:${edge.relation}`"
                :edge="edge"
                :active="active_edge_keys.has(`${edge.from}->${edge.to}`)"
                :dimmed="has_focus && !active_edge_keys.has(`${edge.from}->${edge.to}`)"
              />
              <GraphNodeMesh
                v-for="node in layout.nodes"
                :key="node.id"
                :node="node"
                :selected="selected_id === node.id"
                :hovered="hovered_id === node.id"
                :dimmed="has_focus && !neighbor_ids.has(node.id)"
                @select="on_select"
                @hover="on_hover"
              />
            </TresGroup>
          </TresGroup>
        </TresGroup>

        <TresGridHelper
          :args="[28, 28, '#2a2321', '#1a1514']"
          :position="[controls_target[0], layout.bounds.min.y - 2.2, controls_target[2]]"
          :rotation="[0, 0, 0]"
        />
      </TresCanvas>
    </div>

    <div class="hud">
      <p class="brand_mark">Rgain</p>
      <h1>{{ source_graph.meta.title }}</h1>
      <p class="desc">{{ source_graph.meta.summary }}</p>
      <div class="graph_stats">
        <span>{{ source_graph.meta.axis }}</span>
        <span>{{ layout.nodes.length }} nodes</span>
        <span>{{ layout.edges.length }} edges</span>
        <span v-if="is_demo">demo</span>
      </div>
      <p class="hint">滚轮：绕 X 旋转支路 · 拖拽：观察视角</p>
      <p class="status">Sidecar：{{ sidecar_status }}</p>
    </div>
  </section>
</template>

<style scoped>
.scene {
  position: relative;
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  background: var(--color_night);
}

.canvas_host {
  position: absolute;
  inset: 0;
}

.hud {
  position: absolute;
  left: 1rem;
  top: 1rem;
  z-index: 2;
  max-width: min(22rem, 70%);
  pointer-events: none;
}

.brand_mark {
  margin: 0;
  font-size: 0.72rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--color_brand);
}

h1 {
  margin: 0.25rem 0 0.4rem;
  font-size: 1.15rem;
  font-weight: 650;
  color: var(--color_on_dark);
}

.desc {
  margin: 0;
  color: var(--color_on_dark_soft);
  font-size: 0.78rem;
  line-height: 1.4;
}

.graph_stats {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin-top: 0.65rem;
  font-size: 0.7rem;
  color: var(--color_accent);
}

.graph_stats span {
  padding: 0.15rem 0.45rem;
  border: 1px solid rgb(var(--bright_rgb) / 0.16);
  border-radius: 999px;
  background: rgb(var(--ink_rgb) / 0.35);
}

.hint {
  margin: 0.55rem 0 0;
  font-size: 0.7rem;
  color: var(--color_on_dark_soft);
}

.status {
  display: inline-block;
  margin: 0.55rem 0 0;
  padding: 0.28rem 0.65rem;
  border-radius: 999px;
  border: 1px solid rgb(var(--bright_rgb) / 0.18);
  background: rgb(var(--ink_rgb) / 0.45);
  font-size: 0.72rem;
  color: var(--color_on_dark);
}
</style>
