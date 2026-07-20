import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const adminRetrievalModule = {
  id: "admin-retrieval",
  surface: "backend-admin",
  route: "retrieval",
  titleKey: "memory.admin-retrieval.title",
  descriptionKey: "memory.admin-retrieval.description",
  permission: "memory.backend.indexes.read",
  resources: ["indexes","retrievalProfiles","retrievalTraces"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminRetrievalModule;
