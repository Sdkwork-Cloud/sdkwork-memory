import type { MemoryLearningJob } from './memory-learning-job';
import type { PageInfo } from './page-info';

export interface MemoryLearningJobList {
  items: MemoryLearningJob[];
  pageInfo: PageInfo;
}
