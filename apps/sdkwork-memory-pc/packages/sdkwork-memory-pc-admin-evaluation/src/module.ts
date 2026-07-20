import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const adminEvaluationModule = {
  id: "admin-evaluation",
  surface: "backend-admin",
  route: "evaluation",
  titleKey: "memory.admin-evaluation.title",
  descriptionKey: "memory.admin-evaluation.description",
  permission: "memory.backend.evalRuns.read",
  resources: ["evalRuns"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminEvaluationModule;
