import type { MemorySubject } from './memory-subject';
import type { PageInfo } from './page-info';

export interface MemorySubjectList {
  items: MemorySubject[];
  pageInfo: PageInfo;
}
