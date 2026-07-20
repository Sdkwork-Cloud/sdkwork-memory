import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const adminGovernanceModule = {
  id: "admin-governance",
  surface: "backend-admin",
  route: "governance",
  titleKey: "memory.admin-governance.title",
  descriptionKey: "memory.admin-governance.description",
  permission: "memory.backend.auditLogs.read",
  resources: ["policies","policyAssignments","auditLogs","retentionJobs","migrationJobs"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminGovernanceModule;
