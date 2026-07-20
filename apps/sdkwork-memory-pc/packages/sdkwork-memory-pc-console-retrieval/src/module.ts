import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const consoleRetrievalModule = {
  id: "console-retrieval",
  surface: "app-console",
  route: "retrieval",
  titleKey: "memory.console-retrieval.title",
  descriptionKey: "memory.console-retrieval.description",
  permission: "memory.retrievals.write",
  resources: ["retrievals","contextPacks","feedback"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = consoleRetrievalModule;
