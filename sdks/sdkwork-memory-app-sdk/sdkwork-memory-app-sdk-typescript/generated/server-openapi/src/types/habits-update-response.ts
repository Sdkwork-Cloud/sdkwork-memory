import type { MemoryHabit } from './memory-habit';

export interface HabitsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
