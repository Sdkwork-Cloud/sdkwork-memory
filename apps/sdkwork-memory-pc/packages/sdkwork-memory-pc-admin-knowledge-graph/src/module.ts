import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/knowledge-graph/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/knowledge-graph/module.ts";

export const adminKnowledgeGraphModule = {
  id: "admin-knowledge-graph",
  surface: "backend-admin",
  route: "knowledge-graph",
  titleKey: "memory.admin-knowledge-graph.title",
  descriptionKey: "memory.admin-knowledge-graph.description",
  permission: "memory.backend.entities.read",
  resources: ["entities","edges"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminKnowledgeGraphModule;
