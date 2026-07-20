import type { MemoryEdge } from './memory-edge';

export interface EdgesUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
