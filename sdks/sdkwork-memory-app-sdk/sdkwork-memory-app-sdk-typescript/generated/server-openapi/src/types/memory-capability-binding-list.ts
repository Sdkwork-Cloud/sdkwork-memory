import type { MemoryCapabilityBinding } from './memory-capability-binding';
import type { PageInfo } from './page-info';

export interface MemoryCapabilityBindingList {
  items: MemoryCapabilityBinding[];
  pageInfo: PageInfo;
}
