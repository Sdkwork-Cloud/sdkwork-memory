import type { MemoryExportJob } from './memory-export-job';

export interface ExportJobsCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
