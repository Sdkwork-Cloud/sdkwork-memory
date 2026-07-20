export interface MemoryMigrationJobRequest {
  sourceImplementationProfileId: string;
  targetImplementationProfileId: string;
  mode: 'shadow' | 'dual_write' | 'backfill' | 'cutover' | 'rollback';
  reason: string;
  spaceIds?: string[] | null;
  dryRun?: boolean;
  metadata?: Record<string, unknown> | null;
}
