import type { MemoryLearningSettings } from './memory-learning-settings';

export interface LearningSettingsUpdateResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
