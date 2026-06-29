import type { MemoryHabit } from './memory-habit';

export interface HabitsConfirmResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
