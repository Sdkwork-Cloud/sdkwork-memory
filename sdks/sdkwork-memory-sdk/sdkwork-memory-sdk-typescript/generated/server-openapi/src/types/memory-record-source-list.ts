import type { MemoryPageInfo } from './memory-page-info';
import type { MemoryRecordSource } from './memory-record-source';

export interface MemoryRecordSourceList {
  items: MemoryRecordSource[];
  pageInfo: MemoryPageInfo;
}
