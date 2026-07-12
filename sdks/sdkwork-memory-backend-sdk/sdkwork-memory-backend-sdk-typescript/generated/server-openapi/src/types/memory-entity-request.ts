export interface MemoryEntityRequest {
  tenantId: string;
  spaceId: string;
  entityType: string;
  canonicalName: string;
  aliases?: string[] | null;
  attributes?: Record<string, unknown> | null;
  sensitivityLevel?: string;
}
