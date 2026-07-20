import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/retrieval/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/retrieval/module.ts";

export const consoleRetrievalModule = {
  id: "console-retrieval",
  surface: "app-console",
  route: "retrieval",
  titleKey: "memory.console-retrieval.title",
  descriptionKey: "memory.console-retrieval.description",
  permission: "memory.retrievals.write",
  resources: ["retrievals","contextPacks","feedback"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = consoleRetrievalModule;
