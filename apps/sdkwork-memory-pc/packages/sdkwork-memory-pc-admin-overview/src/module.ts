import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/overview/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/overview/module.ts";

export const adminOverviewModule = {
  id: "admin-overview",
  surface: "backend-admin",
  route: "overview",
  titleKey: "memory.admin-overview.title",
  descriptionKey: "memory.admin-overview.description",
  permission: "memory.backend.commercialReadiness.read",
  resources: ["providerHealth","commercialReadiness"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminOverviewModule;
