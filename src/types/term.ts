export type ExplainTermResult = {
  id: number;
  term: string;
  context: string;
  explanation: string;
};

export type TermExplanationRow = {
  id: number;
  run_id: number | null;
  term: string;
  context: string | null;
  explanation: string;
  created_at: string;
};
