import type { MemoryProviderHealth } from './memory-provider-health';

export interface ProviderHealthRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
