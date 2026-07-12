import type { MemoryPolicyAssignment } from './memory-policy-assignment';

export interface PolicyAssignmentsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
