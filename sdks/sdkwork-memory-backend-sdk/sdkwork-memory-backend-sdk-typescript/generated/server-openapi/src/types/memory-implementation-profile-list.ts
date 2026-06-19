import type { MemoryImplementationProfile } from './memory-implementation-profile';
import type { MemoryPageInfo } from './memory-page-info';

export interface MemoryImplementationProfileList {
  items: MemoryImplementationProfile[];
  pageInfo: MemoryPageInfo;
}
