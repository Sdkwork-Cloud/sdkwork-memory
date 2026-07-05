import type { MemoryBinding } from './memory-binding';
import type { MemoryPageInfo } from './memory-page-info';

export interface MemoryBindingList {
  items: MemoryBinding[];
  pageInfo: MemoryPageInfo;
}
