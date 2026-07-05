import type { MemorySubject } from './memory-subject';
import type { PageInfo } from './page-info';

export interface SubjectsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
