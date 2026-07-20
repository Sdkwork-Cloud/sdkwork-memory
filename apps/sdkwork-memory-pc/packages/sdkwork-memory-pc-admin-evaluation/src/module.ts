import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/evaluation/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/evaluation/module.ts";

export const adminEvaluationModule = {
  id: "admin-evaluation",
  surface: "backend-admin",
  route: "evaluation",
  titleKey: "memory.admin-evaluation.title",
  descriptionKey: "memory.admin-evaluation.description",
  permission: "memory.backend.evalRuns.read",
  resources: ["evalRuns"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminEvaluationModule;
