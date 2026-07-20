import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const consoleOverviewModule = {
  id: "console-overview",
  surface: "app-console",
  route: "overview",
  titleKey: "memory.console-overview.title",
  descriptionKey: "memory.console-overview.description",
  permission: "memory.spaces.read",
  resources: ["spaces","candidates","habits"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = consoleOverviewModule;
