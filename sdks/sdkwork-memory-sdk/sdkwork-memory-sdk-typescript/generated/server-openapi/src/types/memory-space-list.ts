import type { MemorySpace } from './memory-space';
import type { PageInfo } from './page-info';

export interface MemorySpaceList {
  items: MemorySpace[];
  pageInfo: PageInfo;
}
