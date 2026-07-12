import type { MemoryEntity } from './memory-entity';
import type { PageInfo } from './page-info';

export interface MemoryEntityList {
  items: MemoryEntity[];
  pageInfo: PageInfo;
}
