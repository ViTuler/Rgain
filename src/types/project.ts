import type { GraphPayload } from "./graph";

export type ProjectRow = {
  id: number;
  root_path: string;
  kind: string;
  title: string | null;
};

export type RunRow = {
  id: number;
  project_id: number | null;
  prompt: string | null;
  model: string | null;
  status: string;
  created_at: string;
  /** History display title; seeded from graph meta.title. */
  title: string | null;
};

export type IngestResult = {
  project_id: number;
  run_id: number;
  status: string;
  files_processed: number;
  files_cached: number;
  /** LLM passes used in this ingest session (1 = single shot). */
  chunks_processed?: number;
  graph: GraphPayload | null;
  error: string | null;
  warnings: string[];
};
