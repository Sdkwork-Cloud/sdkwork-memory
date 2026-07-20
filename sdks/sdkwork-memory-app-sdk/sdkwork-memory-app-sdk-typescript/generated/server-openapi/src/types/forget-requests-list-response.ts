import type { MemoryForgetJob } from './memory-forget-job';
import type { PageInfo } from './page-info';

export interface ForgetRequestsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
