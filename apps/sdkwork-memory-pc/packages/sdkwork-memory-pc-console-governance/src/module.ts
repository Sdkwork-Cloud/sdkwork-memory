import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const consoleGovernanceModule = {
  id: "console-governance",
  surface: "app-console",
  route: "governance",
  titleKey: "memory.console-governance.title",
  descriptionKey: "memory.console-governance.description",
  permission: "memory.app.policies.write",
  resources: ["policyAssignments","forgetRequests","exportJobs"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = consoleGovernanceModule;
