import type { MemoryPageInfo } from './memory-page-info';
import type { MemoryRetrievalTrace } from './memory-retrieval-trace';

export interface MemoryRetrievalTraceList {
  items: MemoryRetrievalTrace[];
  pageInfo: MemoryPageInfo;
}
