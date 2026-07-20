export interface MemoryAdminSessionContext {
  operatorId?: string;
  permissionScope: readonly string[];
  tenantId?: string;
}
