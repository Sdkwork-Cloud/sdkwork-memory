import type { MemoryRecord } from './memory-record';

export interface MemoriesRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
