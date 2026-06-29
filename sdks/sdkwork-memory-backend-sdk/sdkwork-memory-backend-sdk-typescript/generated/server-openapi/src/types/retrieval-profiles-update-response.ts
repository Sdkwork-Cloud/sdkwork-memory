import type { MemoryRetrievalProfile } from './memory-retrieval-profile';

export interface RetrievalProfilesUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
