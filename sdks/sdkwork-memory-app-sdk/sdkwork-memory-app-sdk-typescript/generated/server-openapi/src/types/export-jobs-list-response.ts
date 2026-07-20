import type { MemoryExportJob } from './memory-export-job';
import type { PageInfo } from './page-info';

export interface ExportJobsListResponse {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
