import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const consoleMemoryModule = {
  id: "console-memory",
  surface: "app-console",
  route: "memory",
  titleKey: "memory.console-memory.title",
  descriptionKey: "memory.console-memory.description",
  permission: "memory.records.read",
  resources: ["spaces","memories"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = consoleMemoryModule;
