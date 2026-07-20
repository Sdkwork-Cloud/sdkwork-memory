import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const adminKnowledgeGraphModule = {
  id: "admin-knowledge-graph",
  surface: "backend-admin",
  route: "knowledge-graph",
  titleKey: "memory.admin-knowledge-graph.title",
  descriptionKey: "memory.admin-knowledge-graph.description",
  permission: "memory.backend.entities.read",
  resources: ["entities","edges"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminKnowledgeGraphModule;
