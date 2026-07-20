import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { messages as enUS } from "./i18n/en-US/memory/learning/module.ts";
import { messages as zhCN } from "./i18n/zh-CN/memory/learning/module.ts";

export const adminLearningModule = {
  id: "admin-learning",
  surface: "backend-admin",
  route: "learning",
  titleKey: "memory.admin-learning.title",
  descriptionKey: "memory.admin-learning.description",
  permission: "memory.backend.candidates.read",
  resources: ["candidates","extractionJobs","consolidationJobs"],
  messages: { "en-US": enUS, "zh-CN": zhCN },
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminLearningModule;
