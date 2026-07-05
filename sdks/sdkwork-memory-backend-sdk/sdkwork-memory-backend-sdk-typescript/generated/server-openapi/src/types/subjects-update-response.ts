import type { MemorySubject } from './memory-subject';

export interface SubjectsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
