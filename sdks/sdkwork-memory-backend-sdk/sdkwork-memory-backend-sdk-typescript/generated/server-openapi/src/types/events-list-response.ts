import type { MemoryEvent } from './memory-event';
import type { PageInfo } from './page-info';

export interface EventsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
