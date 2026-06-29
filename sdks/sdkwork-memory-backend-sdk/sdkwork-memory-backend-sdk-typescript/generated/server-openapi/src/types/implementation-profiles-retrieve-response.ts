import type { MemoryImplementationProfile } from './memory-implementation-profile';

export interface ImplementationProfilesRetrieveResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
