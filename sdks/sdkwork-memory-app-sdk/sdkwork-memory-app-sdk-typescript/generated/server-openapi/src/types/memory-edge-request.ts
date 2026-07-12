export interface MemoryEdgeRequest {
  tenantId: string;
  spaceId: string;
  sourceEntityId: string;
  targetEntityId: string;
  relationType: string;
  weight?: number | null;
  validFrom?: string | null;
  validTo?: string | null;
  metadata?: Record<string, unknown> | null;
}
