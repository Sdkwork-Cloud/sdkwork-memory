import type { MemoryPageInfo } from './memory-page-info';
import type { MemoryRecord } from './memory-record';

export interface MemoryRecordList {
  items: MemoryRecord[];
  pageInfo: MemoryPageInfo;
}
