import type { MemoryCommercialReadiness } from './memory-commercial-readiness';

export interface CommercialReadinessRebuildResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
