import type { MemoryRecord } from './memory-record';
import type { PageInfo } from './page-info';

export interface MemoryRecordList {
  items: MemoryRecord[];
  pageInfo: PageInfo;
}
