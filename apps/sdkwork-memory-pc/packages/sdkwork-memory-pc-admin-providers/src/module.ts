import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/providers/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/providers/module.ts";

export const adminProvidersModule = {
  id: "admin-providers",
  surface: "backend-admin",
  route: "providers",
  titleKey: "memory.admin-providers.title",
  descriptionKey: "memory.admin-providers.description",
  permission: "memory.backend.providerBindings.read",
  resources: ["implementationProfiles","providerBindings","providerHealth"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminProvidersModule;
