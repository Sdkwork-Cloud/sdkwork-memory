import type { MemoryRetrievalResult } from './memory-retrieval-result';

export interface RetrievalsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
