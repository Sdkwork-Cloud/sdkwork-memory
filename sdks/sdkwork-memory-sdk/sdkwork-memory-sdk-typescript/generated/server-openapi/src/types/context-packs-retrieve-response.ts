import type { MemoryContextPack } from './memory-context-pack';

export interface ContextPacksRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
