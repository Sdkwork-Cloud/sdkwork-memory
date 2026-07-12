import type { MemoryPolicy } from './memory-policy';

export interface PoliciesRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
