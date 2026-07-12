export interface MemoryPolicyPatch {
  policyType?: string;
  scope?: string;
  scopeRef?: string | null;
  policy?: Record<string, unknown> | null;
  status?: string;
}
