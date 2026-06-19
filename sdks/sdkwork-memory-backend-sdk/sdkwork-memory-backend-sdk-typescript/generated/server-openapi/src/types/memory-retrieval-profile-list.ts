import type { MemoryPageInfo } from './memory-page-info';
import type { MemoryRetrievalProfile } from './memory-retrieval-profile';

export interface MemoryRetrievalProfileList {
  items: MemoryRetrievalProfile[];
  pageInfo: MemoryPageInfo;
}
