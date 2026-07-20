import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/overview/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/overview/module.ts";

export const consoleOverviewModule = {
  id: "console-overview",
  surface: "app-console",
  route: "overview",
  titleKey: "memory.console-overview.title",
  descriptionKey: "memory.console-overview.description",
  permission: "memory.spaces.read",
  resources: ["spaces","candidates","habits"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = consoleOverviewModule;
