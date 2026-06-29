import type { MemoryRecord } from './memory-record';

export interface MemoriesUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
