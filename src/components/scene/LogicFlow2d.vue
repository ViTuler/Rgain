<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import cytoscape, { type Core, type ElementDefinition, type NodeSingular } from "cytoscape";
import elk from "cytoscape-elk";
import type { Flow2dLayoutPreset, GraphNode, GraphPayload } from "../../types/graph";
import { GRAPH_LAYOUT_VIEW_2D } from "../../types/graph";
import {
    elk_options_for_preset,
    to_cytoscape_elements,
} from "../../lib/graph_adapters";
import {
    build_positions_map,
    clear_graph_layout,
    collect_positions_from_cy,
    load_graph_layout,
    missing_ratio,
    save_graph_layout,
} from "../../lib/graph_layout_cache";
import { demo_graph } from "../../lib/layout_trunk";
import { use_settings } from "../../composables/use_settings";
import { use_theme } from "../../composables/use_theme";
import { DEFAULT_GRAPH_FONT_PX, clamp_font_px } from "../../types/settings";

cytoscape.use(elk);

const { theme } = use_theme();
const { settings } = use_settings();

const props = withDefaults(
    defineProps<{
        graph: GraphPayload | null;
        run_id?: number | null;
        layout_preset?: Flow2dLayoutPreset;
        selected_id?: string | null;
        tag_match_ids?: string[] | null;
    }>(),
    {
        run_id: null,
        layout_preset: "flow-right",
        selected_id: null,
        tag_match_ids: null,
    },
);

const emit = defineEmits<{
    select_node: [node: GraphNode];
    hover_node: [node_id: string | null];
}>();

const host = ref<HTMLElement | null>(null);
let cy: Core | null = null;
let resize_observer: ResizeObserver | null = null;
let save_timer: ReturnType<typeof setTimeout> | null = null;
let hover_clear_timer: ReturnType<typeof setTimeout> | null = null;
let hover_node_id: string | null = null;
let applying_layout = false;
let mount_seq = 0;

/** Delay before clearing hover so moving across gaps/edges does not flash. */
const HOVER_CLEAR_MS = 90;

const source_graph = () => props.graph ?? demo_graph();

function build_stylesheet() {
    const accent = get_css("--color_accent", "#75eec5");
    const brand = get_css("--color_brand", "#ff1b1b");
    const ink = get_css("--color_ink", "#171414");
    const line = get_css("--color_line", "#e7dfdf");
    const surface = get_css("--color_surface", "#f7f3f3");
    const surface2 = get_css("--color_surface_2", "#efe8e8");
    const surface3 = get_css("--color_surface_3", "#fdfbfb");
    const muted = get_css("--color_muted", "#656565");
    const selected_bg = mix_hex(accent, surface, 0.14);
    const label_px = clamp_font_px(settings.value.graph_font_px, DEFAULT_GRAPH_FONT_PX);
    const compound_px = Math.max(10, Math.round((label_px * 11) / 12));
    const edge_px = Math.max(9, Math.round((label_px * 10) / 12));
    const label_width = Math.round((120 * label_px) / 12);

    return [
        {
            selector: "node",
            style: {
                "background-color": surface3,
                "border-width": 1.5,
                "border-color": line,
                label: "data(label)",
                color: ink,
                "font-size": label_px,
                "font-weight": 500,
                "text-valign": "center",
                "text-halign": "center",
                "text-wrap": "wrap",
                "text-max-width": label_width,
                width: "label",
                height: "label",
                padding: "10px",
                shape: "round-rectangle",
            },
        },
        {
            selector: "node[color]",
            style: {
                "border-color": "data(color)",
            },
        },
        {
            selector: "node.trunk",
            style: {
                "border-color": accent,
                "border-width": 2,
            },
        },
        {
            selector: "node.branch",
            style: {
                "border-color": brand,
            },
        },
        {
            selector: "node.compound",
            style: {
                "background-color": surface2,
                "background-opacity": 0.72,
                "border-width": 1,
                "border-color": line,
                "border-style": "dashed",
                "text-valign": "top",
                "text-halign": "center",
                "text-margin-y": -10,
                "text-margin-x": 8,
                color: muted,
                "font-size": compound_px,
                "font-weight": 600,
                padding: "18px",
                shape: "round-rectangle",
            },
        },
        {
            // Fill only — keep trunk/branch border colors so selection
            // does not recolor a branch as if it were a trunk.
            selector: "node:selected",
            style: {
                "background-color": selected_bg,
            },
        },
        {
            selector: "edge",
            style: {
                width: 1.4,
                "curve-style": "bezier",
                "target-arrow-shape": "triangle",
                "target-arrow-color": muted,
                "line-color": muted,
                "arrow-scale": 0.85,
                label: "data(label)",
                "font-size": edge_px,
                color: muted,
                "text-background-color": surface,
                "text-background-opacity": 0.9,
                "text-background-padding": "2px",
                // Let pointer events pass through edges so hover does not
                // flicker when the cursor crosses a link between nodes.
                events: "no",
            },
        },
        {
            selector: "edge.highlight",
            style: {
                width: 2.2,
                "line-color": accent,
                "target-arrow-color": accent,
                "z-index": 9,
            },
        },
        {
            selector: "node.content.hover",
            style: {
                "border-width": 2.2,
            },
        },
        {
            selector: "node.tag_dim",
            style: {
                opacity: 0.2,
            },
        },
        {
            selector: "edge.tag_dim",
            style: {
                opacity: 0.12,
            },
        },
    ] as cytoscape.StylesheetStyle[];
}

function apply_theme_style() {
    if (!cy) return;
    cy.style().fromJson(build_stylesheet()).update();
    sync_selection_style();
}

function mix_hex(a: string, b: string, t: number): string {
    const pa = hex_to_rgb(a);
    const pb = hex_to_rgb(b);
    if (!pa || !pb) return b;
    const r = Math.round(pa.r * t + pb.r * (1 - t));
    const g = Math.round(pa.g * t + pb.g * (1 - t));
    const bl = Math.round(pa.b * t + pb.b * (1 - t));
    return `#${[r, g, bl].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
}

function hex_to_rgb(hex: string): { r: number; g: number; b: number } | null {
    const m = /^#([0-9a-fA-F]{6})$/.exec(hex.trim());
    if (!m) return null;
    const n = parseInt(m[1], 16);
    return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 };
}

function get_css(name: string, fallback: string): string {
    if (typeof window === "undefined") return fallback;
    const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return v || fallback;
}

function schedule_save() {
    if (applying_layout || !cy) return;
    if (save_timer) clearTimeout(save_timer);
    save_timer = setTimeout(() => {
        save_timer = null;
        void persist_layout();
    }, 400);
}

async function persist_layout() {
    if (!cy || applying_layout) return;
    const layout = collect_positions_from_cy(cy, props.layout_preset || "flow-right");
    try {
        await save_graph_layout(props.run_id ?? null, layout, GRAPH_LAYOUT_VIEW_2D);
    } catch (err) {
        console.warn("save_graph_layout failed", err);
    }
}

function run_elk_layout() {
    if (!cy) return;
    applying_layout = true;
    const preset = props.layout_preset || "flow-right";
    const elk_opts = elk_options_for_preset(preset);
    const layout = cy.layout({
        name: "elk",
        fit: true,
        padding: 36,
        animate: false,
        nodeDimensionsIncludeLabels: true,
        elk: elk_opts,
    } as cytoscape.LayoutOptions);

    layout.one("layoutstop", () => {
        cy?.fit(undefined, 36);
        applying_layout = false;
        sync_selection_style();
    });
    layout.run();
}

function run_preset_layout(
    positions: Record<string, { x: number; y: number }>,
    zoom?: number,
    pan?: { x: number; y: number },
) {
    if (!cy) return;
    applying_layout = true;
    const layout = cy.layout({
        name: "preset",
        fit: false,
        animate: false,
        positions: (node: NodeSingular) => positions[node.id()] ?? { x: 0, y: 0 },
    } as cytoscape.LayoutOptions);

    layout.one("layoutstop", () => {
        if (!cy) return;
        if (typeof zoom === "number" && Number.isFinite(zoom)) {
            cy.zoom(zoom);
        }
        if (pan && Number.isFinite(pan.x) && Number.isFinite(pan.y)) {
            cy.pan(pan);
        } else {
            cy.fit(undefined, 36);
        }
        applying_layout = false;
        sync_selection_style();
    });
    layout.run();
}

async function apply_layout_or_elk() {
    if (!cy) return;
    const ids = cy.nodes().map((n) => n.id());
    const cache = await load_graph_layout(props.run_id ?? null, GRAPH_LAYOUT_VIEW_2D);
    if (cache && missing_ratio(cache, ids) <= 0.3) {
        const positions = build_positions_map(cache, ids);
        run_preset_layout(positions, cache.zoom, cache.pan);
        return;
    }
    run_elk_layout();
}

async function mount_cy() {
    if (!host.value) return;
    const seq = ++mount_seq;
    if (save_timer) {
        clearTimeout(save_timer);
        save_timer = null;
    }
    clear_hover_highlight();
    hover_node_id = null;
    cy?.destroy();
    cy = null;

    const graph = source_graph();
    const elements = to_cytoscape_elements(graph) as ElementDefinition[];

    cy = cytoscape({
        container: host.value,
        elements,
        style: build_stylesheet(),
        minZoom: 0.2,
        maxZoom: 2.5,
        wheelSensitivity: 1.5,
        boxSelectionEnabled: false,
        autoungrabify: false,
        autolock: false,
    });

    cy.on("tap", "node.content", (ev) => {
        const id = ev.target.id();
        const node = graph.nodes.find((n) => n.id === id);
        if (node) emit("select_node", node);
    });

    cy.on("tap", (ev) => {
        if (ev.target === cy) {
            emit("hover_node", null);
            clear_hover_highlight();
        }
    });

    cy.on("mouseover", "node.content", (ev) => {
        if (hover_clear_timer) {
            clearTimeout(hover_clear_timer);
            hover_clear_timer = null;
        }
        const id = ev.target.id() as string;
        emit("hover_node", id);
        set_hover_highlight(id);
    });

    cy.on("mouseout", "node.content", () => {
        if (hover_clear_timer) clearTimeout(hover_clear_timer);
        hover_clear_timer = setTimeout(() => {
            hover_clear_timer = null;
            emit("hover_node", null);
            set_hover_highlight(null);
        }, HOVER_CLEAR_MS);
    });

    cy.on("dragfree", "node", () => {
        schedule_save();
    });

    cy.on("zoom", () => {
        schedule_save();
    });

    cy.on("pan", () => {
        schedule_save();
    });

    if (seq !== mount_seq) return;
    sync_tag_filter();
    await apply_layout_or_elk();
}

/**
 * Edge-only hover focus: no full-graph opacity dim (that caused strobing
 * when mouseover/mouseout flipped all elements on every move).
 */
function set_hover_highlight(node_id: string | null) {
    if (!cy) return;
    if (hover_node_id === node_id) return;

    if (hover_node_id) {
        cy.getElementById(hover_node_id).removeClass("hover");
    }
    cy.edges().removeClass("highlight");
    hover_node_id = node_id;

    if (!node_id) return;
    const node = cy.getElementById(node_id);
    if (node.empty()) return;
    node.addClass("hover");
    node.connectedEdges().addClass("highlight");
}

function clear_hover_highlight() {
    if (hover_clear_timer) {
        clearTimeout(hover_clear_timer);
        hover_clear_timer = null;
    }
    set_hover_highlight(null);
}

function sync_tag_filter() {
    const graph = cy;
    if (!graph) return;
    const ids = props.tag_match_ids;
    graph.batch(() => {
        graph.elements().removeClass("tag_dim");
        if (!ids) return;
        const allow = new Set(ids);
        graph.nodes(".content").forEach((node) => {
            if (!allow.has(node.id())) node.addClass("tag_dim");
        });
        graph.edges().forEach((edge) => {
            const from_ok = allow.has(edge.source().id());
            const to_ok = allow.has(edge.target().id());
            if (!from_ok || !to_ok) edge.addClass("tag_dim");
        });
    });
}

function sync_selection_style() {
    if (!cy) return;
    cy.nodes().unselect();
    if (props.selected_id) {
        const n = cy.getElementById(props.selected_id);
        if (!n.empty()) n.select();
    }
}

function fit_view() {
    cy?.fit(undefined, 36);
}

async function reset_layout() {
    await clear_graph_layout(props.run_id ?? null, GRAPH_LAYOUT_VIEW_2D);
    run_elk_layout();
}

/** Clear cached coords (e.g. after switching ELK algorithm) then remount. */
async function clear_cache_and_relayout() {
    await clear_graph_layout(props.run_id ?? null, GRAPH_LAYOUT_VIEW_2D);
    await mount_cy();
}

defineExpose({ fit_view, reset_layout, clear_cache_and_relayout, run_layout: run_elk_layout });

onMounted(() => {
    void mount_cy();
    if (host.value) {
        resize_observer = new ResizeObserver(() => {
            cy?.resize();
        });
        resize_observer.observe(host.value);
    }
});

onBeforeUnmount(() => {
    if (save_timer) {
        clearTimeout(save_timer);
        save_timer = null;
    }
    clear_hover_highlight();
    resize_observer?.disconnect();
    resize_observer = null;
    cy?.destroy();
    cy = null;
});

watch(
    () => [props.graph, props.run_id] as const,
    () => {
        void mount_cy();
    },
);

watch(
    () => props.layout_preset,
    async (next, prev) => {
        if (prev === undefined || next === prev) return;
        await clear_cache_and_relayout();
    },
);

watch(
    () => props.selected_id,
    () => {
        sync_selection_style();
    },
);

watch(
    () => props.tag_match_ids,
    () => {
        sync_tag_filter();
    },
);

watch(theme, () => {
    apply_theme_style();
});

watch(
    () => settings.value.graph_font_px,
    () => {
        apply_theme_style();
    },
);
</script>

<template>
    <div ref="host" class="flow2d_host" role="img" aria-label="2D logic flow graph" />
</template>

<style scoped>
.flow2d_host {
    position: absolute;
    inset: 0;
    background:
        radial-gradient(circle at 1px 1px, color-mix(in srgb, var(--color_line) 70%, transparent) 1px, transparent 0) 0 0 / 18px 18px,
        var(--color_surface);
}
</style>
