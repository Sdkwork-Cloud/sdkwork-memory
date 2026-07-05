export interface MemorySubjectPatch {
  displayName?: string;
  defaultSpaceId?: string | null;
  status?: string;
  metadata?: Record<string, unknown> | null;
}
