import type { MemoryFeedback } from './memory-feedback';

export interface FeedbackCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
