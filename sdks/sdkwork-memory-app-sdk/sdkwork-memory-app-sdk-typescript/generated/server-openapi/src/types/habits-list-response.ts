import type { MemoryHabit } from './memory-habit';
import type { PageInfo } from './page-info';

export interface HabitsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
