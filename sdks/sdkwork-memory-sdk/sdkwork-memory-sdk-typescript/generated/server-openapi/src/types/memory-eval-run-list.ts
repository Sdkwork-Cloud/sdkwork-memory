import type { MemoryEvalRun } from './memory-eval-run';
import type { MemoryPageInfo } from './memory-page-info';

export interface MemoryEvalRunList {
  items: MemoryEvalRun[];
  pageInfo: MemoryPageInfo;
}
