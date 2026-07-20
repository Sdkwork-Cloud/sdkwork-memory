import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const consoleKnowledgeModule = {
  id: "console-knowledge",
  surface: "app-console",
  route: "knowledge",
  titleKey: "memory.console-knowledge.title",
  descriptionKey: "memory.console-knowledge.description",
  permission: "memory.app.entities.read",
  resources: ["entities"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = consoleKnowledgeModule;
