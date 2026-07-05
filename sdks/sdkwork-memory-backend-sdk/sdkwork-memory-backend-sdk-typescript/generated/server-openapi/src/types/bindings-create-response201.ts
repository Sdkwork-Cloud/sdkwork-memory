import type { MemoryBinding } from './memory-binding';

export interface BindingsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
