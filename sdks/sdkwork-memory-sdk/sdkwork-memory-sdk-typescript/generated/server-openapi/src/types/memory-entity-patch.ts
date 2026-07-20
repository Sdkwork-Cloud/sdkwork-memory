export interface MemoryEntityPatch {
  canonicalName?: string;
  aliases?: string[] | null;
  attributes?: Record<string, unknown> | null;
  sensitivityLevel?: string;
  status?: string;
}
