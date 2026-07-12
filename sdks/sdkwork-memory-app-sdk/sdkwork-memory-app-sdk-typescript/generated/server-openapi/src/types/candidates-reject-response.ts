import type { MemoryCandidate } from './memory-candidate';

export interface CandidatesRejectResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
