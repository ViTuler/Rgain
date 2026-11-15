import { invoke } from "@tauri-apps/api/core";
import type { Flow2dLayoutPreset, GraphLayoutCache } from "../types/graph";
import { GRAPH_LAYOUT_VIEW_2D } from "../types/graph";

const memory_by_key = new Map<string, GraphLayoutCache>();

function memory_key(run_id: number | null, view_key: string): string {
    return run_id == null ? `demo:${view_key}` : `run:${run_id}:${view_key}`;
}

type CyLike = {
    nodes: () => {
        forEach: (
            fn: (n: { id: () => string; position: () => { x: number; y: number } }) => void,
        ) => void;
    };
    zoom: () => number;
    pan: () => { x: number; y: number };
};

export function collect_positions_from_cy(
    cy: CyLike,
    preset: Flow2dLayoutPreset,
): GraphLayoutCache {
    const nodes: Record<string, { x: number; y: number }> = {};
    cy.nodes().forEach((n) => {
        const p = n.position();
        nodes[n.id()] = { x: p.x, y: p.y };
    });
    return {
        version: 1,
        preset,
        zoom: cy.zoom(),
        pan: { ...cy.pan() },
        nodes,
    };
}

export function missing_ratio(
    cache: GraphLayoutCache,
    ids: string[],
): number {
    if (!ids.length) return 1;
    let miss = 0;
    for (const id of ids) {
        if (!cache.nodes[id]) miss += 1;
    }
    return miss / ids.length;
}

export function build_positions_map(
    cache: GraphLayoutCache,
    ids: string[],
): Record<string, { x: number; y: number }> {
    const positions: Record<string, { x: number; y: number }> = {};
    for (const id of ids) {
        const p = cache.nodes[id];
        if (p) positions[id] = { x: p.x, y: p.y };
    }

    const missing = ids.filter((id) => !positions[id]);
    if (!missing.length) return positions;

    let ax = 0;
    let ay = 0;
    let n = 0;
    for (const p of Object.values(positions)) {
        ax += p.x;
        ay += p.y;
        n += 1;
    }
    const cx = n ? ax / n : 0;
    const cy = n ? ay / n : 0;
    missing.forEach((id, i) => {
        positions[id] = {
            x: cx + (i % 5) * 48,
            y: cy + Math.floor(i / 5) * 48,
        };
    });
    return positions;
}

export async function load_graph_layout(
    run_id: number | null,
    view_key: string = GRAPH_LAYOUT_VIEW_2D,
): Promise<GraphLayoutCache | null> {
    const key = memory_key(run_id, view_key);
    if (run_id == null) {
        return memory_by_key.get(key) ?? null;
    }
    try {
        const value = await invoke<GraphLayoutCache | null>("get_graph_layout", {
            args: { run_id, view_key },
        });
        if (value?.nodes && typeof value.nodes === "object") {
            memory_by_key.set(key, value);
            return value;
        }
        return null;
    } catch {
        return memory_by_key.get(key) ?? null;
    }
}

export async function save_graph_layout(
    run_id: number | null,
    layout: GraphLayoutCache,
    view_key: string = GRAPH_LAYOUT_VIEW_2D,
): Promise<void> {
    const key = memory_key(run_id, view_key);
    memory_by_key.set(key, layout);
    if (run_id == null) return;
    await invoke("save_graph_layout", {
        args: {
            run_id,
            view_key,
            layout_json: JSON.stringify(layout),
        },
    });
}

export async function clear_graph_layout(
    run_id: number | null,
    view_key: string = GRAPH_LAYOUT_VIEW_2D,
): Promise<void> {
    const key = memory_key(run_id, view_key);
    memory_by_key.delete(key);
    if (run_id == null) return;
    try {
        await invoke("clear_graph_layout", {
            args: { run_id, view_key },
        });
    } catch {
        // ignore when DB unavailable
    }
}
