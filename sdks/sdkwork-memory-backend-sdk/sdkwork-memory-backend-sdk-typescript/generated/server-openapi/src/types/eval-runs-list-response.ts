import type { MemoryEvalRun } from './memory-eval-run';
import type { PageInfo } from './page-info';

export interface EvalRunsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
