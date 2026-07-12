export interface MemoryPolicyAssignmentPatch {
  priority?: number;
  inheritanceMode?: 'inherit' | 'override' | 'deny' | 'shadow';
  status?: string;
  validFrom?: string | null;
  validTo?: string | null;
}
