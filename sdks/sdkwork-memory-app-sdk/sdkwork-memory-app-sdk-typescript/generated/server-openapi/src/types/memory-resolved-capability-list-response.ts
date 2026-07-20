import type { MemoryResolvedCapabilityList } from './memory-resolved-capability-list';

export interface MemoryResolvedCapabilityListResponse {
  code: 0;
  data: unknown & MemoryResolvedCapabilityList;
  /** Server-owned request correlation id. */
  traceId: string;
}
