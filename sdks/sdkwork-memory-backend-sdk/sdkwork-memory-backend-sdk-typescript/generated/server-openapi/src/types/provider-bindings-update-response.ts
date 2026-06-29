import type { MemoryProviderBinding } from './memory-provider-binding';

export interface ProviderBindingsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
