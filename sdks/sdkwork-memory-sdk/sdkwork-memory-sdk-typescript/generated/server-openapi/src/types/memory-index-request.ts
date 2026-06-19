export interface MemoryIndexRequest {
  spaceId?: string | null;
  indexKind: string;
  implementationProfileId?: string | null;
  providerBindingId?: string | null;
  schemaVersion: string;
  config?: Record<string, unknown> | null;
  status?: string;
  version?: string | null;
}
