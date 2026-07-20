import type { MemoryPolicy } from './memory-policy';
import type { PageInfo } from './page-info';

export interface MemoryPolicyList {
  items: MemoryPolicy[];
  pageInfo: PageInfo;
}
