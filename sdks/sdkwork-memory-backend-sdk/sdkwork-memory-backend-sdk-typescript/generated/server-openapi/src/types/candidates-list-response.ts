import type { MemoryCandidate } from './memory-candidate';
import type { PageInfo } from './page-info';

export interface CandidatesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
