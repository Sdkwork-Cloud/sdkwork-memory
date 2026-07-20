import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/knowledge/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/knowledge/module.ts";

export const consoleKnowledgeModule = {
  id: "console-knowledge",
  surface: "app-console",
  route: "knowledge",
  titleKey: "memory.console-knowledge.title",
  descriptionKey: "memory.console-knowledge.description",
  permission: "memory.app.entities.read",
  resources: ["entities"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = consoleKnowledgeModule;
