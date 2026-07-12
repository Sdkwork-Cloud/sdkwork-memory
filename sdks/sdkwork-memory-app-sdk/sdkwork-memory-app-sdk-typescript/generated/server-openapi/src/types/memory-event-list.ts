import type { MemoryEvent } from './memory-event';
import type { PageInfo } from './page-info';

export interface MemoryEventList {
  items: MemoryEvent[];
  pageInfo: PageInfo;
}
