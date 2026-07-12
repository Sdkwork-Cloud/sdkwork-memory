import type { MemoryAuditLog } from './memory-audit-log';
import type { PageInfo } from './page-info';

export interface MemoryAuditLogList {
  items: MemoryAuditLog[];
  pageInfo: PageInfo;
}
