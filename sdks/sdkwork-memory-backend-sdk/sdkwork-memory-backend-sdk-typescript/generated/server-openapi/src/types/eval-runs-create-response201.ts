import type { MemoryEvalRun } from './memory-eval-run';

export interface EvalRunsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
