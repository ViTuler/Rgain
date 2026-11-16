<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import MarkdownBody from "../chrome/MarkdownBody.vue";
import { use_i18n } from "../../i18n";
import type { GraphNode, GraphTagFilter } from "../../types/graph";

const WIDTH_KEY = "rgain_explain_width";
const MIN_WIDTH = 280;
const MAX_WIDTH = 840;

function clamp_width(width: number): number {
    const max = Math.min(MAX_WIDTH, Math.floor(window.innerWidth * 0.78));
    const min = Math.min(MIN_WIDTH, max);
    return Math.min(max, Math.max(min, Math.round(width)));
}

function read_width(): number {
    try {
        const raw = Number(localStorage.getItem(WIDTH_KEY));
        if (Number.isFinite(raw)) return clamp_width(raw);
    } catch {
        // private mode or blocked storage
    }
    return clamp_width(320);
}

const props = defineProps<{
    open: boolean;
    selected_label: string;
    selected_node: GraphNode | null;
    graph_title: string;
    tag_filter: GraphTagFilter | null;
}>();

const emit = defineEmits<{
    toggle_tag: [filter: GraphTagFilter];
    clear_tag: [];
}>();

const { t } = use_i18n();
const panel_width = ref(read_width());
let stop_drag: (() => void) | null = null;

function on_window_resize() {
    panel_width.value = clamp_width(panel_width.value);
}

function on_resize_start(event: PointerEvent) {
    if (!props.open || event.button !== 0) return;
    const current = event.currentTarget;
    if (!(current instanceof HTMLElement)) return;
    const handle = current;
    event.preventDefault();
    handle.setPointerCapture(event.pointerId);
    const start_x = event.clientX;
    const start_width = panel_width.value;
    const previous_cursor = document.body.style.cursor;
    const previous_select = document.body.style.userSelect;
    handle.classList.add("dragging");
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";

    function on_move(ev: PointerEvent) {
        panel_width.value = clamp_width(start_width + (start_x - ev.clientX));
    }

    function finish(ev: PointerEvent) {
        if (handle.hasPointerCapture(ev.pointerId)) handle.releasePointerCapture(ev.pointerId);
        handle.classList.remove("dragging");
        handle.removeEventListener("pointermove", on_move);
        handle.removeEventListener("pointerup", finish);
        handle.removeEventListener("pointercancel", finish);
        document.body.style.cursor = previous_cursor;
        document.body.style.userSelect = previous_select;
        stop_drag = null;
        try {
            localStorage.setItem(WIDTH_KEY, String(panel_width.value));
        } catch {
            // ignore storage failures
        }
    }

    stop_drag = () => finish(new PointerEvent("pointercancel"));
    handle.addEventListener("pointermove", on_move);
    handle.addEventListener("pointerup", finish);
    handle.addEventListener("pointercancel", finish);
}

onMounted(() => {
    window.addEventListener("resize", on_window_resize);
});

onBeforeUnmount(() => {
    window.removeEventListener("resize", on_window_resize);
    stop_drag?.();
});

function tag_on(filter: GraphTagFilter): boolean {
    const current = props.tag_filter;
    if (!current || current.field !== filter.field) return false;
    return current.value === filter.value;
}
</script>

<template>
    <aside
        class="explain_drawer"
        :class="{ open }"
        :style="{ width: `${panel_width}px` }"
        :aria-hidden="!open"
        aria-labelledby="explain_title"
    >
        <div
            class="resize_handle"
            role="separator"
            aria-orientation="vertical"
            :aria-label="t('explain.resize')"
            :aria-valuenow="panel_width"
            :aria-valuemin="MIN_WIDTH"
            :aria-valuemax="MAX_WIDTH"
            @pointerdown="on_resize_start"
        />
        <div class="drawer_scroll">
        <header class="panel_head">
            <span class="eyebrow">{{ t("explain.eyebrow") }}</span>
            <strong id="explain_title">{{ selected_label || t("explain.none_selected") }}</strong>
            <span v-if="graph_title" class="graph_title">{{ graph_title }}</span>
        </header>

        <section class="block">
            <h2>{{ t("explain.section") }}</h2>
            <template v-if="selected_node">
                <p class="meta_row">
                    <button
                        type="button"
                        class="chip"
                        :data-kind="selected_node.kind"
                        :class="{ active: tag_on({ field: 'kind', value: selected_node.kind }) }"
                        :aria-pressed="tag_on({ field: 'kind', value: selected_node.kind })"
                        :title="t('explain.filter_kind')"
                        @click="emit('toggle_tag', { field: 'kind', value: selected_node.kind })"
                    >
                        {{ selected_node.kind }}
                    </button>
                    <button
                        type="button"
                        class="chip"
                        :class="{ active: tag_on({ field: 'module', value: selected_node.module_id }) }"
                        :aria-pressed="tag_on({ field: 'module', value: selected_node.module_id })"
                        :title="t('explain.filter_module')"
                        @click="emit('toggle_tag', { field: 'module', value: selected_node.module_id })"
                    >
                        {{ selected_node.module_id }}
                    </button>
                    <button
                        type="button"
                        class="chip"
                        :class="{ active: tag_on({ field: 'order', value: selected_node.axis_order }) }"
                        :aria-pressed="tag_on({ field: 'order', value: selected_node.axis_order })"
                        :title="t('explain.filter_order')"
                        @click="emit('toggle_tag', { field: 'order', value: selected_node.axis_order })"
                    >
                        {{ t("explain.order", { n: selected_node.axis_order }) }}
                    </button>
                    <span v-if="selected_node.shared" class="chip" data-kind="branch">
                        {{ t("explain.shared") }}
                    </span>
                    <button
                        v-if="tag_filter"
                        type="button"
                        class="chip clear"
                        @click="emit('clear_tag')"
                    >
                        {{ t("explain.filter_clear") }}
                    </button>
                </p>
                <dl v-if="selected_node.story" class="story_facts">
                    <template v-if="selected_node.story.time">
                        <dt>{{ t("explain.story_time") }}</dt>
                        <dd>{{ selected_node.story.time }}</dd>
                    </template>
                    <template v-if="selected_node.story.place">
                        <dt>{{ t("explain.story_place") }}</dt>
                        <dd>{{ selected_node.story.place }}</dd>
                    </template>
                    <template v-if="selected_node.story.actors?.length">
                        <dt>{{ t("explain.story_actors") }}</dt>
                        <dd>{{ selected_node.story.actors.join(" · ") }}</dd>
                    </template>
                    <template v-if="selected_node.story.impact">
                        <dt>{{ t("explain.story_impact") }}</dt>
                        <dd>{{ selected_node.story.impact }}</dd>
                    </template>
                </dl>
                <MarkdownBody class="explain_md" size="explain" :source="selected_node.explain" />
                <div v-if="selected_node.source_refs?.length" class="refs">
                    <h3>{{ t("explain.source_refs") }}</h3>
                    <ul>
                        <li v-for="(ref, i) in selected_node.source_refs" :key="`${ref.path}-${i}`">
                            <code>{{ ref.path }}</code>
                            <span v-if="ref.span" class="span">{{ ref.span }}</span>
                        </li>
                    </ul>
                </div>
            </template>
            <template v-else>
                <p class="muted">{{ t("explain.empty_hint") }}</p>
                <div class="empty_box">{{ t("explain.empty_box") }}</div>
            </template>
        </section>
        </div>
    </aside>
</template>

<style scoped>
.explain_drawer {
    position: absolute;
    /* Sit below stage toolbar so mode/layout controls stay clickable. */
    top: var(--stage_toolbar_height);
    right: 0;
    bottom: 0;
    z-index: 15;
    display: flex;
    flex-direction: column;
    width: min(var(--panel_right_width), 92vw);
    border-left: 1px solid var(--color_line);
    border-top: 1px solid var(--color_line);
    background: var(--color_surface);
    box-shadow: -12px 0 36px -18px rgb(var(--ink_rgb) / 0.35);
    transform: translateX(100%);
    transition: transform 0.24s var(--ease_soft);
    pointer-events: none;
    overflow: hidden;
}

.drawer_scroll {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space_md);
    padding: var(--space_md);
    padding-left: calc(var(--space_md) + 4px);
    overflow: auto;
}

.explain_drawer.open {
    transform: translateX(0);
    pointer-events: auto;
}

.resize_handle {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    z-index: 2;
    width: 6px;
    touch-action: none;
    cursor: col-resize;
}

.resize_handle:hover,
.resize_handle.dragging {
    background: color-mix(in srgb, var(--color_accent) 55%, transparent);
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

.explain_md {
    margin-top: 0.5rem;
}

.story_facts {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.2rem 0.6rem;
    margin: 0.55rem 0 0;
    font-size: 0.78rem;
}

.story_facts dt {
    margin: 0;
    color: var(--color_muted);
}

.story_facts dd {
    margin: 0;
    color: var(--color_body);
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
    font: inherit;
    font-size: 0.68rem;
    line-height: inherit;
    color: var(--color_muted);
    background: var(--color_surface_2);
    cursor: pointer;
}

.chip:focus-visible {
    outline: 1px solid var(--color_accent_deep);
    outline-offset: 1px;
}

.chip[data-kind="trunk"] {
    border-color: color-mix(in srgb, var(--color_accent) 45%, var(--color_line));
    color: var(--color_accent_deep);
}

.chip[data-kind="branch"] {
    border-color: color-mix(in srgb, var(--color_brand) 45%, var(--color_line));
    color: var(--color_brand);
}

.chip.active,
.chip.clear {
    border-color: var(--color_accent_deep);
    color: var(--color_accent_ink);
    background: var(--color_accent_soft);
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
