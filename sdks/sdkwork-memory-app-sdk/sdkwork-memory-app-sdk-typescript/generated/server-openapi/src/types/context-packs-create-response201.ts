import type { MemoryContextPack } from './memory-context-pack';

export interface ContextPacksCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
