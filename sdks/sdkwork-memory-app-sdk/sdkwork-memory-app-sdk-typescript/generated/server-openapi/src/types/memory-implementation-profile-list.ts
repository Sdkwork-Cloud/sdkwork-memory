import type { MemoryImplementationProfile } from './memory-implementation-profile';
import type { PageInfo } from './page-info';

export interface MemoryImplementationProfileList {
  items: MemoryImplementationProfile[];
  pageInfo: PageInfo;
}
