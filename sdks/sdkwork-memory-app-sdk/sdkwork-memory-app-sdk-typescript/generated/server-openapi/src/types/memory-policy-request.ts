export interface MemoryPolicyRequest {
  tenantId: string;
  policyType: string;
  scope: string;
  scopeRef?: string | null;
  policy: Record<string, unknown> | null;
}
