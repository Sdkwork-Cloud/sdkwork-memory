import type { MemoryEntity } from './memory-entity';
import type { PageInfo } from './page-info';

export interface EntitiesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
