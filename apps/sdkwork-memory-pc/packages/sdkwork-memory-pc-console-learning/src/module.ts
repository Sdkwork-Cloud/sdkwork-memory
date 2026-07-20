import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const consoleLearningModule = {
  id: "console-learning",
  surface: "app-console",
  route: "learning",
  titleKey: "memory.console-learning.title",
  descriptionKey: "memory.console-learning.description",
  permission: "memory.candidates.read",
  resources: ["candidates","habits","learningSettings"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = consoleLearningModule;
