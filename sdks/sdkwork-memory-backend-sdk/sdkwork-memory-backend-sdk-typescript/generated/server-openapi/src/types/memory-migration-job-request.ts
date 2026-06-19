export interface MemoryMigrationJobRequest {
  sourceImplementationProfileId: string;
  targetImplementationProfileId: string;
  mode: 'shadow' | 'dual_write' | 'backfill' | 'cutover' | 'rollback';
  spaceIds?: string[] | null;
  dryRun?: boolean;
  metadata?: Record<string, unknown> | null;
}
