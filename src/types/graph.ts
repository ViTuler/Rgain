/**
 * Canonical logic-graph contract (LLM → SQLite → Vue).
 *
 * Persistence & Rust validation stay on this shape.
 * View layers never mutate it; they project via adapters:
 *   - 2D flow  → `to_cytoscape_elements` (Cytoscape.js / ELK, json-to-graph style)
 *   - 3D trunk → `layout_trunk_graph` (Tres / Three)
 *
 * Cytoscape element mapping (tabkram/json-to-graph):
 *   node  → { group: "nodes", data: { id, label, parent?, color? } }
 *   edge  → { group: "edges", data: { id, source, target, label? } }
 *   parent = visual compound box (group_id), NOT trunk parent_id.
 */

export type GraphAxis = "time" | "logic" | "story";

export type GraphNodeKind = "trunk" | "branch";

export type GraphEdgeRelation = "next" | "derives" | "contrasts" | string;

/** UI / layout projection mode. Independent of stored payload. */
export type LogicViewMode = "2d" | "3d";

/**
 * 2D ELK presets aligned with json-to-graph layout picker.
 * 3D uses trunk timeline separately.
 */
export type Flow2dLayoutPreset =
    | "flow-right"
    | "flow-down"
    | "tree"
    | "radial"
    | "force";

export type GraphModule = {
    id: string;
    name: string;
    /** Hex #rrggbb preferred; used by both 2D border and 3D fill. */
    color_hint?: string;
};

export type SourceRef = {
    path: string;
    span?: string;
};

export type GraphNode = {
    id: string;
    label: string;
    module_id: string;
    /** Narrative / process order along the main axis. */
    axis_order: number;
    kind: GraphNodeKind;
    /**
     * Semantic parent for trunk/branch (logic attachment).
     * Used by 3D trunk layout; optional hint for 2D edge emphasis.
     */
    parent_id: string | null;
    /**
     * Visual compound parent for 2D nesting (Cytoscape `data.parent`).
     * When set, maps to a group node; must reference a `GraphGroup.id`
     * or another content node used as a box. Null = top-level.
     */
    group_id?: string | null;
    explain: string;
    source_refs?: SourceRef[];
    /** Optional per-node accent; falls back to module color_hint. */
    color?: string;
};

/**
 * Compound / nest box for 2D flow (json-to-graph style containers).
 * Not drawn as meshes in 3D trunk (modules still color 3D nodes).
 */
export type GraphGroup = {
    id: string;
    label: string;
    /** Nest groups: Cytoscape parent chain. */
    parent_id?: string | null;
    color?: string;
};

export type GraphEdge = {
    from: string;
    to: string;
    relation: GraphEdgeRelation;
};

export type GraphTerm = {
    term: string;
    node_ids: string[];
};

export type GraphPayload = {
    meta: {
        title: string;
        axis: GraphAxis;
        summary: string;
        /** Preferred opening view; UI may override. Default 2d. */
        default_view?: LogicViewMode;
        /** Preferred 2D layout; default flow-right. */
        default_flow_layout?: Flow2dLayoutPreset;
    };
    modules: GraphModule[];
    /** Optional compound boxes for 2D nesting. */
    groups?: GraphGroup[];
    nodes: GraphNode[];
    edges: GraphEdge[];
    terms?: GraphTerm[];
};

/** Selection event shared by 2D and 3D stages. */
export type GraphSelection = {
    node_id: string;
    node: GraphNode;
};

/** Persisted 2D (or future 3D) node positions — not part of GraphPayload. */
export type GraphLayoutCache = {
    version: 1;
    preset?: Flow2dLayoutPreset;
    zoom?: number;
    pan?: { x: number; y: number };
    nodes: Record<string, { x: number; y: number }>;
};

export const GRAPH_LAYOUT_VIEW_2D = "2d-flow";
