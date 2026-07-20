import type { MemoryEdge } from './memory-edge';
import type { PageInfo } from './page-info';

export interface EdgesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
