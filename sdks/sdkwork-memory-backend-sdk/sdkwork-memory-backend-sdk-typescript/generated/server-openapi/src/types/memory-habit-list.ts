import type { MemoryHabit } from './memory-habit';
import type { MemoryPageInfo } from './memory-page-info';

export interface MemoryHabitList {
  items: MemoryHabit[];
  pageInfo: MemoryPageInfo;
}
