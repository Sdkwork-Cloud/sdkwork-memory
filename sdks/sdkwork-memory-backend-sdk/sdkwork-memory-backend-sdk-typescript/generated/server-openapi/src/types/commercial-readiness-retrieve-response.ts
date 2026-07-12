import type { MemoryCommercialReadiness } from './memory-commercial-readiness';

export interface CommercialReadinessRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
