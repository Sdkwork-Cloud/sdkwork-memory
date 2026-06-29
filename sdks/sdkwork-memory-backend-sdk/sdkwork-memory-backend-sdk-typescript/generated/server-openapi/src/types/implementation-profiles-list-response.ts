import type { MemoryImplementationProfile } from './memory-implementation-profile';
import type { PageInfo } from './page-info';

export interface ImplementationProfilesListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
