import type { MemoryProviderBinding } from './memory-provider-binding';
import type { PageInfo } from './page-info';

export interface MemoryProviderBindingList {
  items: MemoryProviderBinding[];
  pageInfo: PageInfo;
}
