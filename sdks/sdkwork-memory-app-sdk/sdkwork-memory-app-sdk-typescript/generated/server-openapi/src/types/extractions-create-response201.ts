import type { MemoryLearningJob } from './memory-learning-job';

export interface ExtractionsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
