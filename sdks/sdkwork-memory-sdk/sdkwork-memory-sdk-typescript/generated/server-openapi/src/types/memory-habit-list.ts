import type { MemoryHabit } from './memory-habit';
import type { PageInfo } from './page-info';

export interface MemoryHabitList {
  items: MemoryHabit[];
  pageInfo: PageInfo;
}
