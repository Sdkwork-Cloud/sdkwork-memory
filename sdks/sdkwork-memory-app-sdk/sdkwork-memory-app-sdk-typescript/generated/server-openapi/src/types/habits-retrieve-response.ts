import type { MemoryHabit } from './memory-habit';

export interface HabitsRetrieveResponse {
  code: 0;
  data: unknown & { item: MemoryHabit; };
  /** Server-owned request correlation id. */
  traceId: string;
}
