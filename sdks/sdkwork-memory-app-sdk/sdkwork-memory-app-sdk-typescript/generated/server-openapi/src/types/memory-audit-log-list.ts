import type { MemoryAuditLog } from './memory-audit-log';
import type { MemoryPageInfo } from './memory-page-info';

export interface MemoryAuditLogList {
  items: MemoryAuditLog[];
  pageInfo: MemoryPageInfo;
}
