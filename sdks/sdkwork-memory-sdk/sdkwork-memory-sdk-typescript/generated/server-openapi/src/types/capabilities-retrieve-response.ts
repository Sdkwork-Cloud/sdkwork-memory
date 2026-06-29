import type { MemoryCapabilities } from './memory-capabilities';

export interface CapabilitiesRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
