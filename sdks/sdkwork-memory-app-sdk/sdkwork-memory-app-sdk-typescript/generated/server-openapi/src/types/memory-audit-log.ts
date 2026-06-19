export interface MemoryAuditLog {
  auditLogId: string;
  actorType: string;
  actorId?: string | null;
  action: string;
  resourceType: string;
  resourceId?: string | null;
  requestId?: string | null;
  traceId?: string | null;
  result: string;
  reason?: string | null;
  metadata?: Record<string, unknown> | null;
  createdAt: string;
}
