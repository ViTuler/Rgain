import type {
    Flow2dLayoutPreset,
    GraphEdge,
    GraphGroup,
    GraphNode,
    GraphPayload,
} from "../types/graph";

/** Cytoscape.js element shape used by json-to-graph / cytoscape-elk. */
export type CytoscapeElement = {
    group: "nodes" | "edges";
    data: Record<string, unknown>;
    classes?: string;
};

export type ElkLayoutOptions = {
    algorithm: string;
    "elk.direction"?: string;
    "elk.spacing.nodeNode"?: number;
    "elk.layered.spacing.nodeNodeBetweenLayers"?: number;
    "elk.padding"?: string;
    [key: string]: string | number | undefined;
};

const MODULE_GROUP_PREFIX = "mod__";

export function module_group_id(module_id: string): string {
    return `${MODULE_GROUP_PREFIX}${module_id}`;
}

export function is_module_group_id(id: string): boolean {
    return id.startsWith(MODULE_GROUP_PREFIX);
}

/**
 * Project canonical GraphPayload → Cytoscape elements.
 *
 * Compound parents (priority):
 *   1. node.group_id → GraphGroup / another node id
 *   2. else wrap content nodes inside a module compound (`mod__{module_id}`)
 *
 * Edges use Cytoscape `source` / `target` (json-to-graph), mapped from `from` / `to`.
 */
export function to_cytoscape_elements(
    graph: GraphPayload,
    options: { nest_by_module?: boolean } = {},
): CytoscapeElement[] {
    const nest_by_module = options.nest_by_module ?? true;
    const elements: CytoscapeElement[] = [];
    const group_ids = new Set((graph.groups ?? []).map((g) => g.id));
    const node_ids = new Set(graph.nodes.map((n) => n.id));

    for (const group of graph.groups ?? []) {
        elements.push(group_to_cy_node(group));
    }

    const color_by_module = new Map(
        graph.modules.map((m) => [m.id, m.color_hint] as const),
    );

    const needed_module_groups = new Set<string>();
    const node_parents = new Map<string, string | null>();

    for (const node of graph.nodes) {
        const parent = resolve_cy_parent(node, group_ids, node_ids, nest_by_module);
        node_parents.set(node.id, parent);
        if (parent && is_module_group_id(parent)) {
            needed_module_groups.add(parent);
        }
    }

    if (nest_by_module) {
        for (const mod of graph.modules) {
            const gid = module_group_id(mod.id);
            if (!needed_module_groups.has(gid)) continue;
            if (group_ids.has(gid) || node_ids.has(gid)) continue;
            elements.push({
                group: "nodes",
                data: {
                    id: gid,
                    label: mod.name,
                    color: mod.color_hint,
                    rgain_kind: "module_group",
                    module_id: mod.id,
                },
                classes: "compound module_box",
            });
        }
    }

    for (const node of graph.nodes) {
        const parent = node_parents.get(node.id);
        const color =
            node.color || color_by_module.get(node.module_id) || undefined;
        elements.push({
            group: "nodes",
            data: {
                id: node.id,
                label: node.label,
                parent: parent ?? undefined,
                color,
                rgain_kind: node.kind,
                module_id: node.module_id,
                axis_order: node.axis_order,
                parent_id: node.parent_id,
                group_id: node.group_id ?? null,
            },
            classes: `content ${node.kind}`,
        });
    }

    for (const edge of graph.edges) {
        elements.push(edge_to_cy(edge));
    }

    return elements;
}

function group_to_cy_node(group: GraphGroup): CytoscapeElement {
    return {
        group: "nodes",
        data: {
            id: group.id,
            label: group.label,
            parent: group.parent_id ?? undefined,
            color: group.color,
            rgain_kind: "group",
        },
        classes: "compound group_box",
    };
}

function resolve_cy_parent(
    node: GraphNode,
    group_ids: Set<string>,
    node_ids: Set<string>,
    nest_by_module: boolean,
): string | null {
    const gid = node.group_id;
    if (gid && (group_ids.has(gid) || node_ids.has(gid))) {
        return gid;
    }
    if (nest_by_module) {
        return module_group_id(node.module_id);
    }
    return null;
}

function edge_to_cy(edge: GraphEdge): CytoscapeElement {
    const id = `${edge.from}->${edge.to}:${edge.relation}`;
    return {
        group: "edges",
        data: {
            id,
            source: edge.from,
            target: edge.to,
            label: edge.relation === "next" ? "" : edge.relation,
            relation: edge.relation,
        },
    };
}

/**
 * Best-effort reverse map from Cytoscape JSON (array or {nodes,edges})
 * into a GraphPayload. Used for importing json-to-graph demos / fixtures.
 */
export function from_cytoscape_elements(
    raw: CytoscapeElement[] | { nodes?: CytoscapeElement[]; edges?: CytoscapeElement[] },
    meta: GraphPayload["meta"],
): GraphPayload {
    const list: CytoscapeElement[] = Array.isArray(raw)
        ? raw
        : [...(raw.nodes ?? []), ...(raw.edges ?? [])];

    const cy_nodes = list.filter((e) => e.group === "nodes" || e.data?.source == null);
    const cy_edges = list.filter(
        (e) => e.group === "edges" || (e.data?.source != null && e.data?.target != null),
    );

    const child_parent = new Map<string, string>();
    for (const el of cy_nodes) {
        const id = String(el.data.id);
        const parent = el.data.parent != null ? String(el.data.parent) : null;
        if (parent) child_parent.set(id, parent);
    }

    const parents = new Set(child_parent.values());
    const groups: GraphGroup[] = [];
    const content_nodes: GraphNode[] = [];
    const modules_map = new Map<string, { id: string; name: string; color_hint?: string }>();

    let order = 1;
    for (const el of cy_nodes) {
        const id = String(el.data.id);
        const label = String(el.data.label ?? id);
        const color =
            typeof el.data.color === "string" ? el.data.color : undefined;
        // Appears as another node's `parent` → compound box.
        if (parents.has(id)) {
            groups.push({
                id,
                label,
                parent_id: child_parent.get(id) ?? null,
                color,
            });
            continue;
        }

        const module_raw =
            typeof el.data.module_id === "string"
                ? el.data.module_id
                : child_parent.get(id) || "default";
        const module_id = is_module_group_id(module_raw)
            ? module_raw.slice(MODULE_GROUP_PREFIX.length)
            : module_raw;

        if (!modules_map.has(module_id)) {
            modules_map.set(module_id, {
                id: module_id,
                name: module_id,
                color_hint: color,
            });
        }

        const kind =
            el.data.rgain_kind === "branch" || el.data.rgain_kind === "trunk"
                ? el.data.rgain_kind
                : "trunk";

        content_nodes.push({
            id,
            label,
            module_id,
            axis_order:
                typeof el.data.axis_order === "number" ? el.data.axis_order : order,
            kind,
            parent_id:
                typeof el.data.parent_id === "string" ? el.data.parent_id : null,
            group_id: child_parent.get(id) ?? null,
            explain:
                typeof el.data.explain === "string"
                    ? el.data.explain
                    : `${label}`,
            color,
        });
        order += 1;
    }

    const edges: GraphEdge[] = cy_edges.map((el) => ({
        from: String(el.data.source),
        to: String(el.data.target),
        relation:
            typeof el.data.relation === "string"
                ? el.data.relation
                : typeof el.data.label === "string" && el.data.label
                  ? String(el.data.label)
                  : "next",
    }));

    return {
        meta,
        modules: [...modules_map.values()],
        groups,
        nodes: content_nodes,
        edges,
    };
}

/** ELK options for each json-to-graph-like preset. */
export function elk_options_for_preset(preset: Flow2dLayoutPreset): ElkLayoutOptions {
    switch (preset) {
        case "flow-down":
            return {
                algorithm: "layered",
                "elk.direction": "DOWN",
                "elk.spacing.nodeNode": 28,
                "elk.layered.spacing.nodeNodeBetweenLayers": 56,
                "elk.padding": "[top=24,left=24,bottom=24,right=24]",
            };
        case "tree":
            return {
                algorithm: "mrtree",
                "elk.direction": "DOWN",
                "elk.spacing.nodeNode": 32,
                "elk.padding": "[top=24,left=24,bottom=24,right=24]",
            };
        case "radial":
            return {
                algorithm: "radial",
                "elk.spacing.nodeNode": 40,
                "elk.padding": "[top=32,left=32,bottom=32,right=32]",
            };
        case "force":
            return {
                algorithm: "force",
                "elk.spacing.nodeNode": 40,
                "elk.padding": "[top=24,left=24,bottom=24,right=24]",
            };
        case "flow-right":
        default:
            return {
                algorithm: "layered",
                "elk.direction": "RIGHT",
                "elk.spacing.nodeNode": 28,
                "elk.layered.spacing.nodeNodeBetweenLayers": 64,
                "elk.padding": "[top=24,left=24,bottom=24,right=24]",
            };
    }
}
