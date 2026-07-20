import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const adminControlPlaneModule = {
  id: "admin-control-plane",
  surface: "backend-admin",
  route: "control-plane",
  titleKey: "memory.admin-control-plane.title",
  descriptionKey: "memory.admin-control-plane.description",
  permission: "memory.backend.subjects.read",
  resources: ["subjects","bindings","capabilityBindings","capabilities"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminControlPlaneModule;
