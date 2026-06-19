export interface MemoryEvalRunRequest {
  evalType: string;
  datasetRef?: string | null;
  profileRef?: string | null;
  config?: Record<string, unknown> | null;
}
