import type { MemoryPageInfo } from './memory-page-info';
import type { MemoryProviderBinding } from './memory-provider-binding';

export interface MemoryProviderBindingList {
  items: MemoryProviderBinding[];
  pageInfo: MemoryPageInfo;
}
