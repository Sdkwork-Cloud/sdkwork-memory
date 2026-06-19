import type { MemoryCandidate } from './memory-candidate';
import type { MemoryPageInfo } from './memory-page-info';

export interface MemoryCandidateList {
  items: MemoryCandidate[];
  pageInfo: MemoryPageInfo;
}
