import type { MemoryRetrievalTrace } from './memory-retrieval-trace';

export interface RetrievalTracesRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
