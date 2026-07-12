import type { MemoryPolicyAssignment } from './memory-policy-assignment';
import type { PageInfo } from './page-info';

export interface MemoryPolicyAssignmentList {
  items: MemoryPolicyAssignment[];
  pageInfo: PageInfo;
}
