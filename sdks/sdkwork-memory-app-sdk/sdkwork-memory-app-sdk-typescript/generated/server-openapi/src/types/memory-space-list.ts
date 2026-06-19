import type { MemoryPageInfo } from './memory-page-info';
import type { MemorySpace } from './memory-space';

export interface MemorySpaceList {
  items: MemorySpace[];
  pageInfo: MemoryPageInfo;
}
