import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const adminMemoryModule = {
  id: "admin-memory",
  surface: "backend-admin",
  route: "memory",
  titleKey: "memory.admin-memory.title",
  descriptionKey: "memory.admin-memory.description",
  permission: "memory.backend.records.read",
  resources: ["spaces","memories","events"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminMemoryModule;
