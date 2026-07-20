import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/memory/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/memory/module.ts";

export const adminMemoryModule = {
  id: "admin-memory",
  surface: "backend-admin",
  route: "memory",
  titleKey: "memory.admin-memory.title",
  descriptionKey: "memory.admin-memory.description",
  permission: "memory.backend.records.read",
  resources: ["spaces","memories","events"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminMemoryModule;
