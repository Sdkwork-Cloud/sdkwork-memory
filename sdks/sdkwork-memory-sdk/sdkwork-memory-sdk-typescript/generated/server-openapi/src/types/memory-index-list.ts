import type { MemoryIndex } from './memory-index';
import type { MemoryPageInfo } from './memory-page-info';

export interface MemoryIndexList {
  items: MemoryIndex[];
  pageInfo: MemoryPageInfo;
}
