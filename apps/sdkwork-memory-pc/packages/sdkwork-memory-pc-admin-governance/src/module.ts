import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/governance/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/governance/module.ts";

export const adminGovernanceModule = {
  id: "admin-governance",
  surface: "backend-admin",
  route: "governance",
  titleKey: "memory.admin-governance.title",
  descriptionKey: "memory.admin-governance.description",
  permission: "memory.backend.auditLogs.read",
  resources: ["policies","policyAssignments","auditLogs","retentionJobs","migrationJobs"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminGovernanceModule;
