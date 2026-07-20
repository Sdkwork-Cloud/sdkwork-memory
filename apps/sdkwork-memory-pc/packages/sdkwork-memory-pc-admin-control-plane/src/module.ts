import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/control-plane/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/control-plane/module.ts";

export const adminControlPlaneModule = {
  id: "admin-control-plane",
  surface: "backend-admin",
  route: "control-plane",
  titleKey: "memory.admin-control-plane.title",
  descriptionKey: "memory.admin-control-plane.description",
  permission: "memory.backend.subjects.read",
  resources: ["subjects","bindings","capabilityBindings","capabilities"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminControlPlaneModule;
