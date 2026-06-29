import type { MemoryForgetJob } from './memory-forget-job';

export interface ForgetRequestsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
