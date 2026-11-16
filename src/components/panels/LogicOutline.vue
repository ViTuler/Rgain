<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import type { OutlineKind, OutlineNode } from "../../lib/logic_outline";

type OutlineRow = {
    id: string;
    label: string;
    kind: OutlineKind;
    depth: number;
    has_children: boolean;
    open: boolean;
};

const props = defineProps<{
    nodes: OutlineNode[];
    selected_id: string | null;
}>();

const emit = defineEmits<{
    select: [id: string];
}>();

const collapsed = ref(new Set<string>());
const row_els = new Map<string, HTMLElement>();

function folder_ids(nodes: OutlineNode[]): Set<string> {
    const ids = new Set<string>();
    function walk(list: OutlineNode[]) {
        for (const node of list) {
            if (node.children.length === 0) continue;
            ids.add(node.id);
            walk(node.children);
        }
    }
    walk(nodes);
    return ids;
}

const parent_of = computed(() => {
    const map = new Map<string, string | null>();
    function walk(list: OutlineNode[], parent_id: string | null) {
        for (const node of list) {
            map.set(node.id, parent_id);
            walk(node.children, node.id);
        }
    }
    walk(props.nodes, null);
    return map;
});

const rows = computed(() => {
    const list: OutlineRow[] = [];
    const closed = collapsed.value;
    function walk(nodes: OutlineNode[], depth: number) {
        for (const node of nodes) {
            const has_children = node.children.length > 0;
            const open = has_children && !closed.has(node.id);
            list.push({
                id: node.id,
                label: node.label,
                kind: node.kind,
                depth,
                has_children,
                open,
            });
            if (open) walk(node.children, depth + 1);
        }
    }
    walk(props.nodes, 0);
    return list;
});

watch(
    () => props.nodes,
    () => {
        collapsed.value = folder_ids(props.nodes);
    },
    { immediate: true },
);

watch(
    () => props.selected_id,
    async (id) => {
        if (!id) return;
        expand_to(id);
        await nextTick();
        row_els.get(id)?.scrollIntoView({ block: "nearest" });
    },
);

function expand_to(id: string) {
    const next = new Set(collapsed.value);
    let parent = parent_of.value.get(id) ?? null;
    const seen = new Set<string>();
    let changed = false;
    while (parent && !seen.has(parent)) {
        seen.add(parent);
        if (next.delete(parent)) changed = true;
        parent = parent_of.value.get(parent) ?? null;
    }
    if (changed) collapsed.value = next;
}

function toggle(id: string) {
    const next = new Set(collapsed.value);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    collapsed.value = next;
}

function bind_row(id: string, el: unknown) {
    if (el instanceof HTMLElement) row_els.set(id, el);
    else row_els.delete(id);
}

function on_row_click(row: OutlineRow) {
    if (row.kind === "module") {
        toggle(row.id);
        return;
    }
    emit("select", row.id);
}
</script>

<template>
    <div class="outline" role="tree">
        <div
            v-for="row in rows"
            :key="row.id"
            :ref="(el) => bind_row(row.id, el)"
            class="row"
            role="treeitem"
            :class="{ active: selected_id === row.id }"
            :aria-selected="selected_id === row.id"
            :aria-expanded="row.has_children ? row.open : undefined"
            :style="{ paddingLeft: `${4 + row.depth * 12}px` }"
            :title="row.label"
            @click="on_row_click(row)"
        >
            <button
                v-if="row.has_children"
                type="button"
                class="twisty"
                :class="{ open: row.open }"
                tabindex="-1"
                @click.stop="toggle(row.id)"
            />
            <span v-else class="twisty spacer" />
            <svg v-if="row.kind === 'trunk'" class="icon" viewBox="0 0 16 16" aria-hidden="true">
                <path fill="#519aba" d="M3.2 1.15h5.85V4.7h3.9v9c0 .5-.4.9-.9.9H4.1c-.5 0-.9-.4-.9-.9V2.05c0-.5.4-.9.9-.9z" />
                <path fill="#d7eef7" d="M9.05 1.15 12.95 4.7H9.05z" />
            </svg>
            <svg v-else-if="row.kind === 'branch'" class="icon" viewBox="0 0 16 16" aria-hidden="true">
                <path fill="#e37933" d="M3.2 1.15h5.85V4.7h3.9v9c0 .5-.4.9-.9.9H4.1c-.5 0-.9-.4-.9-.9V2.05c0-.5.4-.9.9-.9z" />
                <path fill="#fde7d6" d="M9.05 1.15 12.95 4.7H9.05z" />
            </svg>
            <span class="label">{{ row.label }}</span>
        </div>
    </div>
</template>

<style scoped>
.outline {
    padding: 0.15rem 0 0.4rem;
}

.row {
    display: flex;
    align-items: center;
    gap: 0.1rem;
    height: 22px;
    padding-right: 0.45rem;
    color: var(--color_body);
    cursor: pointer;
    user-select: none;
}

.row:hover {
    background: var(--color_surface_2);
}

.row.active {
    background: color-mix(in srgb, var(--color_accent) 32%, var(--color_surface_2));
    color: var(--color_ink);
}

.twisty {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    padding: 0;
    border: 0;
    background: transparent;
    position: relative;
    cursor: pointer;
}

.twisty::before {
    content: "";
    position: absolute;
    left: 5px;
    top: 4px;
    width: 0;
    height: 0;
    border-style: solid;
    border-width: 4px 0 4px 5px;
    border-color: transparent transparent transparent var(--color_faint);
}

.twisty.open::before {
    left: 3px;
    top: 6px;
    border-width: 5px 4px 0 4px;
    border-color: var(--color_faint) transparent transparent transparent;
}

.twisty.spacer::before {
    display: none;
}

.icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
}

.label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    line-height: 22px;
}
</style>
