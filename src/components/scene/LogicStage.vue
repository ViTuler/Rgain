<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type {
    Flow2dLayoutPreset,
    GraphNode,
    GraphPayload,
    LogicViewMode,
} from "../../types/graph";
import { demo_graph } from "../../lib/layout_trunk";
import LogicFlow2d from "./LogicFlow2d.vue";
import LogicScene from "./LogicScene.vue";

const props = defineProps<{
    graph: GraphPayload | null;
    run_id?: number | null;
    sidecar_status: string;
}>();

const emit = defineEmits<{
    select_node: [node: GraphNode];
}>();

const source = computed(() => props.graph ?? demo_graph());
const is_demo = computed(() => !props.graph);
const has_run = computed(() => props.run_id != null);

const view_mode = ref<LogicViewMode>(
    source.value.meta.default_view ?? "2d",
);
const flow_layout = ref<Flow2dLayoutPreset>(
    source.value.meta.default_flow_layout ?? "flow-right",
);
const selected_id = ref<string | null>(null);
const flow2d_ref = ref<InstanceType<typeof LogicFlow2d> | null>(null);

const layout_options: { value: Flow2dLayoutPreset; label: string }[] = [
    { value: "flow-right", label: "Flow →" },
    { value: "flow-down", label: "Flow ↓" },
    { value: "tree", label: "Tree" },
    { value: "radial", label: "Radial" },
    { value: "force", label: "Force" },
];

watch(
    () => props.graph,
    (g) => {
        selected_id.value = null;
        if (g?.meta.default_view) view_mode.value = g.meta.default_view;
        if (g?.meta.default_flow_layout) {
            flow_layout.value = g.meta.default_flow_layout;
        }
    },
);

function on_select(node: GraphNode) {
    selected_id.value = node.id;
    emit("select_node", node);
}

function set_mode(mode: LogicViewMode) {
    view_mode.value = mode;
}

async function on_reset_layout() {
    await flow2d_ref.value?.reset_layout();
}
</script>

<template>
    <section class="stage">
        <div class="stage_toolbar">
            <div class="mode_toggle" role="tablist" aria-label="Logic view mode">
                <button
                    type="button"
                    role="tab"
                    class="mode_btn"
                    :aria-selected="view_mode === '2d'"
                    :class="{ active: view_mode === '2d' }"
                    @click="set_mode('2d')"
                >
                    2D Flow
                </button>
                <button
                    type="button"
                    role="tab"
                    class="mode_btn"
                    :aria-selected="view_mode === '3d'"
                    :class="{ active: view_mode === '3d' }"
                    @click="set_mode('3d')"
                >
                    3D Trunk
                </button>
            </div>

            <label v-if="view_mode === '2d'" class="layout_pick">
                <span class="sr_only">2D layout</span>
                <select v-model="flow_layout" aria-label="2D layout preset">
                    <option
                        v-for="opt in layout_options"
                        :key="opt.value"
                        :value="opt.value"
                    >
                        {{ opt.label }}
                    </option>
                </select>
            </label>

            <button
                v-if="view_mode === '2d'"
                type="button"
                class="reset_btn"
                title="清除已保存位置并重新自动布局"
                @click="on_reset_layout"
            >
                重置布局
            </button>

            <div class="meta_chips">
                <span>{{ source.meta.axis }}</span>
                <span>{{ source.nodes.length }} nodes</span>
                <span>{{ source.edges.length }} edges</span>
                <span v-if="is_demo">demo</span>
                <span v-else-if="has_run">run #{{ run_id }}</span>
            </div>
        </div>

        <div class="stage_body">
            <LogicFlow2d
                v-if="view_mode === '2d'"
                ref="flow2d_ref"
                :graph="graph"
                :run_id="run_id ?? null"
                :layout_preset="flow_layout"
                :selected_id="selected_id"
                @select_node="on_select"
            />
            <LogicScene
                v-else
                :graph="graph"
                :sidecar_status="sidecar_status"
                @select_node="on_select"
            />

            <div v-if="view_mode === '2d'" class="hud_2d">
                <p class="brand_mark">Rgain</p>
                <h1>{{ source.meta.title }}</h1>
                <p class="desc">{{ source.meta.summary }}</p>
                <p class="hint">
                    拖拽节点调整位置会自动保存
                    <template v-if="has_run">到 SQLite</template>
                    <template v-else>（demo 仅内存）</template>
                    · 滚轮缩放 · 拖拽背景平移
                </p>
                <p class="status">Sidecar：{{ sidecar_status }}</p>
            </div>
        </div>
    </section>
</template>

<style scoped>
.stage {
    position: relative;
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
    background: var(--color_surface);
}

.stage_toolbar {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    z-index: 4;
    height: 34px;
    padding: 0 0.65rem;
    border-bottom: 1px solid var(--color_line);
    background: color-mix(in srgb, var(--color_surface) 88%, var(--color_surface_2));
}

.mode_toggle {
    display: inline-flex;
    padding: 2px;
    border: 1px solid var(--color_line);
    border-radius: 8px;
    background: var(--color_surface_2);
}

.mode_btn {
    appearance: none;
    border: 0;
    border-radius: 6px;
    padding: 0.2rem 0.65rem;
    background: transparent;
    color: var(--color_muted);
    font-size: 0.72rem;
    font-weight: 650;
    letter-spacing: 0.02em;
    cursor: pointer;
}

.mode_btn.active {
    background: var(--color_surface_3);
    color: var(--color_ink);
    box-shadow: 0 1px 2px rgb(var(--ink_rgb) / 0.14);
}

.layout_pick select {
    height: 26px;
    padding: 0 0.45rem;
    border: 1px solid var(--color_line);
    border-radius: 6px;
    background: var(--color_surface_3);
    color: var(--color_ink);
    font-size: 0.72rem;
}

.layout_pick select option {
    background: var(--color_surface);
    color: var(--color_ink);
}

.reset_btn {
    appearance: none;
    height: 26px;
    padding: 0 0.55rem;
    border: 1px solid var(--color_line);
    border-radius: 6px;
    background: var(--color_surface_3);
    color: var(--color_muted);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
}

.reset_btn:hover {
    color: var(--color_ink);
    border-color: var(--color_line_strong);
}

.meta_chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-left: auto;
    font-size: 0.68rem;
    color: var(--color_muted);
}

.meta_chips span {
    padding: 0.1rem 0.4rem;
    border: 1px solid var(--color_line);
    border-radius: 999px;
    background: var(--color_surface_3);
    color: var(--color_muted);
}

.stage_body {
    position: relative;
    flex: 1;
    min-height: 0;
}

.stage_body :deep(.scene) {
    position: absolute;
    inset: 0;
}

.hud_2d {
    position: absolute;
    left: 1rem;
    top: 1rem;
    z-index: 2;
    max-width: min(22rem, 55%);
    pointer-events: none;
}

.brand_mark {
    margin: 0;
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--color_brand);
}

.hud_2d h1 {
    margin: 0.25rem 0 0.35rem;
    font-size: 1.05rem;
    font-weight: 650;
    color: var(--color_ink);
}

.desc {
    margin: 0;
    color: var(--color_muted);
    font-size: 0.78rem;
    line-height: 1.4;
}

.hint {
    margin: 0.55rem 0 0;
    font-size: 0.7rem;
    color: var(--color_faint);
}

.status {
    display: inline-block;
    margin: 0.55rem 0 0;
    padding: 0.28rem 0.65rem;
    border-radius: 999px;
    border: 1px solid var(--color_line);
    background: color-mix(in srgb, var(--color_surface_3) 82%, transparent);
    font-size: 0.72rem;
    color: var(--color_body);
}

.sr_only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
}
</style>
