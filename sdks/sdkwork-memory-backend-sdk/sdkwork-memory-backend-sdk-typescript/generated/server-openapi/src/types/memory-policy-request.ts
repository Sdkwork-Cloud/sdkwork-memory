export interface MemoryPolicyRequest {
  policyType: string;
  scope: string;
  scopeRef?: string | null;
  policy: Record<string, unknown> | null;
}
