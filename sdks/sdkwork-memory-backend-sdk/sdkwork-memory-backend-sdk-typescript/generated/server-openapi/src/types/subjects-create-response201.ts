import type { MemorySubject } from './memory-subject';

export interface SubjectsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
