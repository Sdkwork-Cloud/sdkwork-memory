import type { MemoryCapabilityBinding } from './memory-capability-binding';

export interface CapabilityBindingsRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
