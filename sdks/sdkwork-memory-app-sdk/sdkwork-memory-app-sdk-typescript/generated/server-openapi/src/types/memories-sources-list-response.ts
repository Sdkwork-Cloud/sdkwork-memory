import type { MemoryRecordSource } from './memory-record-source';
import type { PageInfo } from './page-info';

export interface MemoriesSourcesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
