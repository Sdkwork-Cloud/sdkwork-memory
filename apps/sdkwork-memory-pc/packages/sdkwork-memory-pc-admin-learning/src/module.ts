import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const adminLearningModule = {
  id: "admin-learning",
  surface: "backend-admin",
  route: "learning",
  titleKey: "memory.admin-learning.title",
  descriptionKey: "memory.admin-learning.description",
  permission: "memory.backend.candidates.read",
  resources: ["candidates","extractionJobs","consolidationJobs"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminLearningModule;
