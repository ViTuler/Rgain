import type { GraphEdge, GraphNode, GraphPayload } from "../types/graph";

export type Vec3 = { x: number; y: number; z: number };

export type LaidOutNode = GraphNode & {
  position: Vec3;
  color: string;
  /** Trunk cylinder half-length along X (full length = 2 * this for mesh). */
  trunk_half_len: number;
};

export type LaidOutEdge = GraphEdge & {
  from_pos: Vec3;
  to_pos: Vec3;
};

export type GraphLayout = {
  nodes: LaidOutNode[];
  edges: LaidOutEdge[];
  bounds: { min: Vec3; max: Vec3; center: Vec3 };
};

const ACCENT = "#75eec5";
const BRAND = "#ff1b1b";
const MODULE_PALETTE = [
  "#6bb3a8",
  "#8a9bb5",
  "#a89b7a",
  "#7a9e8e",
  "#9a8a9e",
  "#7d8f9a",
  "#a0927c",
  "#6e9aa0",
];

/** Main logic extends along +X. */
const AXIS_STEP = 3.2;
/** Branch radius in the YZ plane. */
const BRANCH_R = 2.4;
const MIN_SEP_YZ = 1.25;

function module_color(module_id: string, index: number, hint?: string): string {
  if (hint && /^#[0-9a-fA-F]{6}$/.test(hint)) return hint;
  let hash = 0;
  for (let i = 0; i < module_id.length; i += 1) {
    hash = (hash * 31 + module_id.charCodeAt(i)) >>> 0;
  }
  return MODULE_PALETTE[(hash + index) % MODULE_PALETTE.length];
}

/** Repel branches in YZ when they share a similar X (same trunk band). */
function push_apart_yz(positions: Map<string, Vec3>, ids: string[], kinds: Map<string, string>) {
  for (let iter = 0; iter < 10; iter += 1) {
    for (let i = 0; i < ids.length; i += 1) {
      for (let j = i + 1; j < ids.length; j += 1) {
        if (kinds.get(ids[i]) === "trunk" && kinds.get(ids[j]) === "trunk") continue;
        const a = positions.get(ids[i])!;
        const b = positions.get(ids[j])!;
        if (Math.abs(a.x - b.x) > AXIS_STEP * 0.7) continue;
        const dy = a.y - b.y;
        const dz = a.z - b.z;
        const dist = Math.hypot(dy, dz) || 0.001;
        if (dist >= MIN_SEP_YZ) continue;
        const push = (MIN_SEP_YZ - dist) * 0.5;
        const ny = dy / dist;
        const nz = dz / dist;
        a.y += ny * push;
        a.z += nz * push;
        b.y -= ny * push;
        b.z -= nz * push;
      }
    }
  }
}

/**
 * Trunk Timeline on X:
 * - trunk nodes along +X by axis_order (Y=0, Z=0)
 * - branch nodes in the YZ plane around their parent trunk
 * - wheel later spins the whole graph around X so YZ branches rotate
 */
export function layout_trunk_graph(graph: GraphPayload): GraphLayout {
  const nodes = [...graph.nodes].sort((a, b) => a.axis_order - b.axis_order);
  const by_id = new Map(nodes.map((n) => [n.id, n]));
  const module_index = new Map(graph.modules.map((m, i) => [m.id, i]));
  const positions = new Map<string, Vec3>();
  const kinds = new Map(nodes.map((n) => [n.id, n.kind]));

  const trunks = nodes.filter((n) => n.kind === "trunk");
  const branches = nodes.filter((n) => n.kind === "branch");

  trunks.forEach((node, i) => {
    positions.set(node.id, { x: i * AXIS_STEP, y: 0, z: 0 });
  });

  if (trunks.length === 0) {
    nodes.forEach((node, i) => {
      positions.set(node.id, { x: i * AXIS_STEP, y: 0, z: 0 });
    });
  }

  const trunk_by_order = trunks.length
    ? trunks
    : [...nodes].sort((a, b) => a.axis_order - b.axis_order);

  function nearest_trunk(order: number): GraphNode | null {
    if (!trunk_by_order.length) return null;
    let best = trunk_by_order[0];
    let best_d = Math.abs(best.axis_order - order);
    for (const t of trunk_by_order) {
      const d = Math.abs(t.axis_order - order);
      if (d < best_d) {
        best = t;
        best_d = d;
      }
    }
    return best;
  }

  const side_count = new Map<string, number>();

  for (const node of branches) {
    const parent =
      (node.parent_id && by_id.get(node.parent_id)) ||
      nearest_trunk(node.axis_order);
    const anchor = parent ? positions.get(parent.id) : undefined;
    const base_x = anchor?.x ?? node.axis_order * AXIS_STEP * 0.35;
    const key = parent?.id ?? "_root";
    const slot = side_count.get(key) ?? 0;
    side_count.set(key, slot + 1);
    const count_later = Math.max(slot + 1, 1);
    // Spread on a YZ ring; angle filled as siblings appear.
    const angle = (slot / Math.max(count_later, 3)) * Math.PI * 2 - Math.PI / 2;
    const tier = 1 + Math.floor(slot / 6) * 0.55;
    const r = BRANCH_R * tier;
    positions.set(node.id, {
      x: base_x,
      y: Math.sin(angle) * r,
      z: Math.cos(angle) * r,
    });
  }

  // Re-spread with known sibling counts for even YZ rings.
  side_count.clear();
  const siblings = new Map<string, GraphNode[]>();
  for (const node of branches) {
    const parent =
      (node.parent_id && by_id.get(node.parent_id)) ||
      nearest_trunk(node.axis_order);
    const key = parent?.id ?? "_root";
    const list = siblings.get(key) ?? [];
    list.push(node);
    siblings.set(key, list);
  }
  for (const [key, list] of siblings) {
    const parent = key === "_root" ? null : by_id.get(key) ?? null;
    const base_x = parent ? positions.get(parent.id)?.x ?? 0 : list[0]
      ? (nearest_trunk(list[0].axis_order)
          ? positions.get(nearest_trunk(list[0].axis_order)!.id)?.x ?? 0
          : 0)
      : 0;
    list.forEach((node, slot) => {
      const angle = (slot / list.length) * Math.PI * 2 - Math.PI / 2;
      const r = BRANCH_R * (1 + Math.floor(slot / 8) * 0.4);
      positions.set(node.id, {
        x: base_x,
        y: Math.sin(angle) * r,
        z: Math.cos(angle) * r,
      });
    });
  }

  nodes.forEach((node, i) => {
    if (!positions.has(node.id)) {
      positions.set(node.id, {
        x: i * AXIS_STEP * 0.5,
        y: 0,
        z: 0,
      });
    }
  });

  push_apart_yz(
    positions,
    nodes.map((n) => n.id),
    kinds,
  );

  const trunk_half = AXIS_STEP * 0.38;

  const laid_nodes: LaidOutNode[] = nodes.map((node) => {
    const mi = module_index.get(node.module_id) ?? 0;
    const mod = graph.modules.find((m) => m.id === node.module_id);
    const base = module_color(node.module_id, mi, mod?.color_hint);
    const fill =
      node.kind === "trunk" ? ACCENT : mix_hex(base, BRAND, 0.25);
    return {
      ...node,
      position: positions.get(node.id)!,
      color: fill,
      trunk_half_len: trunk_half,
    };
  });

  const laid_edges: LaidOutEdge[] = graph.edges
    .filter((e) => positions.has(e.from) && positions.has(e.to))
    .map((e) => ({
      ...e,
      from_pos: { ...positions.get(e.from)! },
      to_pos: { ...positions.get(e.to)! },
    }));

  const xs = laid_nodes.map((n) => n.position.x);
  const ys = laid_nodes.map((n) => n.position.y);
  const zs = laid_nodes.map((n) => n.position.z);
  const min = {
    x: Math.min(...xs, 0),
    y: Math.min(...ys, -BRANCH_R),
    z: Math.min(...zs, -BRANCH_R),
  };
  const max = {
    x: Math.max(...xs, 0),
    y: Math.max(...ys, BRANCH_R),
    z: Math.max(...zs, BRANCH_R),
  };
  const center = {
    x: (min.x + max.x) / 2,
    y: (min.y + max.y) / 2,
    z: (min.z + max.z) / 2,
  };

  return { nodes: laid_nodes, edges: laid_edges, bounds: { min, max, center } };
}

function mix_hex(a: string, b: string, t: number): string {
  const pa = hex_to_rgb(a);
  const pb = hex_to_rgb(b);
  if (!pa || !pb) return a;
  const r = Math.round(pa.r + (pb.r - pa.r) * t);
  const g = Math.round(pa.g + (pb.g - pa.g) * t);
  const bl = Math.round(pa.b + (pb.b - pa.b) * t);
  return `#${[r, g, bl].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
}

function hex_to_rgb(hex: string): { r: number; g: number; b: number } | null {
  const m = /^#([0-9a-fA-F]{6})$/.exec(hex);
  if (!m) return null;
  const n = parseInt(m[1], 16);
  return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 };
}

/** Demo graph so the stage is usable before a real ingest. */
export function demo_graph(): GraphPayload {
  return {
    meta: {
      title: "Demo Logic Flow",
      axis: "logic",
      summary: "默认 2D 分层流（Flow →）；可切到 3D Trunk。",
      default_view: "2d",
      default_flow_layout: "flow-right",
    },
    modules: [
      { id: "m_core", name: "Core", color_hint: "#6bb3a8" },
      { id: "m_io", name: "I/O", color_hint: "#8a9bb5" },
    ],
    groups: [
      {
        id: "g_ingest",
        label: "Ingest",
        parent_id: null,
        color: "#6bb3a8",
      },
      {
        id: "g_reason",
        label: "Reason",
        parent_id: null,
        color: "#8a9bb5",
      },
    ],
    nodes: [
      {
        id: "n1",
        label: "入口",
        module_id: "m_core",
        axis_order: 1,
        kind: "trunk",
        parent_id: null,
        group_id: "g_ingest",
        explain: "系统入口与启动路径。",
      },
      {
        id: "n2",
        label: "解析",
        module_id: "m_core",
        axis_order: 2,
        kind: "trunk",
        parent_id: "n1",
        group_id: "g_ingest",
        explain: "抽取文本并组装语料。",
      },
      {
        id: "n2b",
        label: "缓存",
        module_id: "m_io",
        axis_order: 2,
        kind: "branch",
        parent_id: "n2",
        group_id: "g_ingest",
        explain: "按内容哈希跳过重复抽取。",
      },
      {
        id: "n3",
        label: "归纳",
        module_id: "m_core",
        axis_order: 3,
        kind: "trunk",
        parent_id: "n2",
        group_id: "g_reason",
        explain: "LLM 产出 trunk/branch 图 JSON。",
      },
      {
        id: "n3b",
        label: "校验",
        module_id: "m_io",
        axis_order: 3,
        kind: "branch",
        parent_id: "n3",
        group_id: "g_reason",
        explain: "schema 失败时触发一次修复重试。",
      },
      {
        id: "n3c",
        label: "修复",
        module_id: "m_io",
        axis_order: 3,
        kind: "branch",
        parent_id: "n3",
        group_id: "g_reason",
        explain: "一次 repair 重试后的校正路径。",
      },
      {
        id: "n4",
        label: "舞台",
        module_id: "m_core",
        axis_order: 4,
        kind: "trunk",
        parent_id: "n3",
        group_id: null,
        explain: "2D 分层流 / 3D Trunk 讲解舞台。",
      },
    ],
    edges: [
      { from: "n1", to: "n2", relation: "next" },
      { from: "n2", to: "n2b", relation: "derives" },
      { from: "n2", to: "n3", relation: "next" },
      { from: "n3", to: "n3b", relation: "derives" },
      { from: "n3", to: "n3c", relation: "derives" },
      { from: "n3", to: "n4", relation: "next" },
    ],
  };
}
