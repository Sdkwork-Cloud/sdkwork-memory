import type { MemoryCapabilityBinding } from './memory-capability-binding';
import type { MemoryPageInfo } from './memory-page-info';

export interface MemoryCapabilityBindingList {
  items: MemoryCapabilityBinding[];
  pageInfo: MemoryPageInfo;
}
