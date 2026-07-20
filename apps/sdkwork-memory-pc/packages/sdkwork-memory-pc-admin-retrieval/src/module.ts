import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/retrieval/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/retrieval/module.ts";

export const adminRetrievalModule = {
  id: "admin-retrieval",
  surface: "backend-admin",
  route: "retrieval",
  titleKey: "memory.admin-retrieval.title",
  descriptionKey: "memory.admin-retrieval.description",
  permission: "memory.backend.indexes.read",
  resources: ["indexes","retrievalProfiles","retrievalTraces"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminRetrievalModule;
