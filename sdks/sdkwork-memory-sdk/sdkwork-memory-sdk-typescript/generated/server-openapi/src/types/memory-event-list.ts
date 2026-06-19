import type { MemoryEvent } from './memory-event';
import type { MemoryPageInfo } from './memory-page-info';

export interface MemoryEventList {
  items: MemoryEvent[];
  pageInfo: MemoryPageInfo;
}
