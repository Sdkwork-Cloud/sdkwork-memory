import type { MemoryCapabilityBinding } from './memory-capability-binding';
import type { PageInfo } from './page-info';

export interface CapabilityBindingsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
