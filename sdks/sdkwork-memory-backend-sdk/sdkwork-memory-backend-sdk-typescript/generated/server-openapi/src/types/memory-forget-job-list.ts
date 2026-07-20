import type { MemoryForgetJob } from './memory-forget-job';
import type { PageInfo } from './page-info';

export interface MemoryForgetJobList {
  items: MemoryForgetJob[];
  pageInfo: PageInfo;
}
