import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/governance/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/governance/module.ts";

export const consoleGovernanceModule = {
  id: "console-governance",
  surface: "app-console",
  route: "governance",
  titleKey: "memory.console-governance.title",
  descriptionKey: "memory.console-governance.description",
  permission: "memory.app.policies.write",
  resources: ["policyAssignments","forgetRequests","exportJobs"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = consoleGovernanceModule;
