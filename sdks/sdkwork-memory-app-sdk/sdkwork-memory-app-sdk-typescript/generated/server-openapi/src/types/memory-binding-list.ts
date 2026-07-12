import type { MemoryBinding } from './memory-binding';
import type { PageInfo } from './page-info';

export interface MemoryBindingList {
  items: MemoryBinding[];
  pageInfo: PageInfo;
}
