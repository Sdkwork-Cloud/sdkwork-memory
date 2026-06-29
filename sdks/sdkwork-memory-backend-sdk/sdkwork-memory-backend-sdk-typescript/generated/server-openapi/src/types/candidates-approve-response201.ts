import type { MemoryCandidate } from './memory-candidate';

export interface CandidatesApproveResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
