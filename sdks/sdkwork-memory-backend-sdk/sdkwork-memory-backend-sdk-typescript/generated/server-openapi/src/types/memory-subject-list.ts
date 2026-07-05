import type { MemoryPageInfo } from './memory-page-info';
import type { MemorySubject } from './memory-subject';

export interface MemorySubjectList {
  items: MemorySubject[];
  pageInfo: MemoryPageInfo;
}
