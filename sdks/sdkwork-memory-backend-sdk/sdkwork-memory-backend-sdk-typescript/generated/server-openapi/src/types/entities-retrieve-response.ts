import type { MemoryEntity } from './memory-entity';

export interface EntitiesRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
