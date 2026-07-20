import type { MemoryIndex } from './memory-index';
import type { PageInfo } from './page-info';

export interface MemoryIndexList {
  items: MemoryIndex[];
  pageInfo: PageInfo;
}
