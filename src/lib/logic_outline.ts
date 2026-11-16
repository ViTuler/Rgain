import type { GraphNode, GraphNodeKind, GraphPayload } from "../types/graph";

export type OutlineKind = "module" | GraphNodeKind;

export type OutlineNode = {
    id: string;
    label: string;
    kind: OutlineKind;
    children: OutlineNode[];
};

function by_axis(a: GraphNode, b: GraphNode): number {
    if (a.axis_order !== b.axis_order) return a.axis_order - b.axis_order;
    if (a.label !== b.label) return a.label < b.label ? -1 : 1;
    return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
}

/**
 * One module is one folder. Every graph node in that module is a child,
 * in axis order. Trunk nodes are not folders.
 */
export function build_logic_outline(graph: GraphPayload): OutlineNode[] {
    const by_module = new Map<string, GraphNode[]>();
    for (const node of graph.nodes) {
        const list = by_module.get(node.module_id) ?? [];
        list.push(node);
        by_module.set(node.module_id, list);
    }

    const used = new Set<string>();
    const folders: OutlineNode[] = [];

    function push_module(module_id: string, label: string) {
        const nodes = by_module.get(module_id);
        if (!nodes || nodes.length === 0 || used.has(module_id)) return;
        used.add(module_id);
        folders.push({
            id: `module:${module_id}`,
            label,
            kind: "module",
            children: nodes.slice().sort(by_axis).map((node) => ({
                id: node.id,
                label: node.label,
                kind: node.kind,
                children: [],
            })),
        });
    }

    for (const mod of graph.modules) {
        push_module(mod.id, mod.name || mod.id);
    }
    for (const module_id of by_module.keys()) {
        push_module(module_id, module_id);
    }
    return folders;
}
