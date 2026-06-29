import type { MemoryRetrievalProfile } from './memory-retrieval-profile';
import type { PageInfo } from './page-info';

export interface RetrievalProfilesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
