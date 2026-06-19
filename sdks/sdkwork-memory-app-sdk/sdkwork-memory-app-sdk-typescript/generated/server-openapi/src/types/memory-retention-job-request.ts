export interface MemoryRetentionJobRequest {
  scope: string;
  spaceId?: string | null;
  dryRun?: boolean;
  policyRef?: string | null;
  metadata?: Record<string, unknown> | null;
}
