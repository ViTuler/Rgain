export type GraphAxis = "time" | "logic" | "story";

export type GraphModule = {
  id: string;
  name: string;
  color_hint?: string;
};

export type GraphNode = {
  id: string;
  label: string;
  module_id: string;
  axis_order: number;
  kind: "trunk" | "branch";
  parent_id: string | null;
  explain: string;
  source_refs?: { path: string; span?: string }[];
};

export type GraphEdge = {
  from: string;
  to: string;
  relation: "next" | "derives" | "contrasts" | string;
};

export type GraphPayload = {
  meta: {
    title: string;
    axis: GraphAxis;
    summary: string;
  };
  modules: GraphModule[];
  nodes: GraphNode[];
  edges: GraphEdge[];
  terms?: { term: string; node_ids: string[] }[];
};
