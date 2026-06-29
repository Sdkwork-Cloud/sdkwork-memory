import type { MemoryImplementationProfile } from './memory-implementation-profile';

export interface ImplementationProfilesCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
