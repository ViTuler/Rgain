<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type {
    Flow2dLayoutPreset,
    GraphNode,
    GraphPayload,
    GraphTagFilter,
    LogicViewMode,
} from "../../types/graph";
import { node_matches_tag } from "../../types/graph";
import { demo_graph } from "../../lib/layout_trunk";
import { use_i18n } from "../../i18n";
import GraphHudTitle from "./GraphHudTitle.vue";
import LogicFlow2d from "./LogicFlow2d.vue";
import LogicScene from "./LogicScene.vue";

const props = defineProps<{
    graph: GraphPayload | null;
    run_id?: number | null;
    tag_filter?: GraphTagFilter | null;
    focus_node_id?: string | null;
}>();

const { t } = use_i18n();

const emit = defineEmits<{
    select_node: [node: GraphNode];
}>();

const source = computed(() => props.graph ?? demo_graph());
const tag_match_ids = computed(() => {
    const filter = props.tag_filter;
    if (!filter) return null;
    return source.value.nodes
        .filter((node) => node_matches_tag(node, filter))
        .map((node) => node.id);
});
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
        selected_id.value = props.focus_node_id ?? null;
        if (g?.meta.default_view) view_mode.value = g.meta.default_view;
        if (g?.meta.default_flow_layout) {
            flow_layout.value = g.meta.default_flow_layout;
        }
    },
);

watch(
    () => props.focus_node_id,
    (id) => {
        selected_id.value = id ?? null;
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
                <button type="button" role="tab" class="mode_btn" :aria-selected="view_mode === '2d'"
                    :class="{ active: view_mode === '2d' }" @click="set_mode('2d')">
                    {{ t("stage.mode_2d") }}
                </button>
                <button type="button" role="tab" class="mode_btn" :aria-selected="view_mode === '3d'"
                    :class="{ active: view_mode === '3d' }" @click="set_mode('3d')" :disabled="true"
                    :title="t('stage.mode_3d_disabled')">
                    {{ t("stage.mode_3d") }}
                </button>
            </div>

            <label v-if="view_mode === '2d'" class="layout_pick">
                <span class="sr_only">{{ t("stage.layout_preset") }}</span>
                <select v-model="flow_layout" :aria-label="t('stage.layout_preset')">
                    <option v-for="opt in layout_options" :key="opt.value" :value="opt.value">
                        {{ opt.label }}
                    </option>
                </select>
            </label>

            <button v-if="view_mode === '2d'" type="button" class="reset_btn"
                :title="t('stage.reset_layout_title')" @click="on_reset_layout">
                {{ t("stage.reset_layout") }}
            </button>

            <div class="toolbar_right">
                <div class="meta_chips">
                    <span>{{ source.meta.axis }}</span>
                    <span>{{ source.nodes.length }} nodes</span>
                    <span>{{ source.edges.length }} edges</span>
                    <span v-if="is_demo">{{ t("common.demo") }}</span>
                    <span v-else-if="has_run">run #{{ run_id }}</span>
                </div>
                <div class="legend_chips" role="group" :aria-label="t('stage.legend')">
                    <span class="legend_chip" data-kind="trunk">
                        <i class="swatch" aria-hidden="true" />
                        {{ t("stage.legend_trunk") }}
                    </span>
                    <span class="legend_chip" data-kind="branch">
                        <i class="swatch" aria-hidden="true" />
                        {{ t("stage.legend_branch") }}
                    </span>
                </div>
            </div>
        </div>

        <div class="stage_body">
            <LogicFlow2d v-if="view_mode === '2d'" ref="flow2d_ref" :graph="graph" :run_id="run_id ?? null"
                :layout_preset="flow_layout" :selected_id="selected_id" :tag_match_ids="tag_match_ids"
                @select_node="on_select" />
            <LogicScene v-else :graph="graph" :tag_match_ids="tag_match_ids" :selected_id="selected_id"
                @select_node="on_select" />

            <div v-if="view_mode === '2d'" class="hud_2d">
                <GraphHudTitle tone="light" :title="source.meta.title" :summary="source.meta.summary" />
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
    z-index: 16;
    height: var(--stage_toolbar_height);
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

.mode_btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
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

.toolbar_right {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.45rem;
    margin-left: auto;
}

.meta_chips,
.legend_chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    font-size: 0.68rem;
    color: var(--color_muted);
}

.meta_chips span,
.legend_chip {
    display: inline-flex;
    align-items: center;
    gap: 0.28rem;
    padding: 0.1rem 0.4rem;
    border: 1px solid var(--color_line);
    border-radius: 999px;
    background: var(--color_surface_3);
    color: var(--color_muted);
    user-select: none;
    pointer-events: none;
}

.legend_chip .swatch {
    display: block;
    width: 0.45rem;
    height: 0.45rem;
    border-radius: 999px;
    background: currentColor;
}

.legend_chip[data-kind="trunk"] {
    border-color: color-mix(in srgb, var(--color_accent) 45%, var(--color_line));
    color: var(--color_accent_deep);
}

.legend_chip[data-kind="branch"] {
    border-color: color-mix(in srgb, var(--color_brand) 45%, var(--color_line));
    color: var(--color_brand);
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
