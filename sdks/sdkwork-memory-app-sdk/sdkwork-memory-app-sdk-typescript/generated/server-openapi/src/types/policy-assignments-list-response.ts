import type { MemoryPolicyAssignment } from './memory-policy-assignment';
import type { PageInfo } from './page-info';

export interface PolicyAssignmentsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
