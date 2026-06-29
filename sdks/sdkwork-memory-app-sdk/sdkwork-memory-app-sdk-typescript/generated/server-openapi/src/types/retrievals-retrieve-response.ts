import type { MemoryRetrievalResult } from './memory-retrieval-result';

export interface RetrievalsRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
