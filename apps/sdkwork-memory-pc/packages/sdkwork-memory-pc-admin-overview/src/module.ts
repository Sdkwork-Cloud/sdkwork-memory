import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const adminOverviewModule = {
  id: "admin-overview",
  surface: "backend-admin",
  route: "overview",
  titleKey: "memory.admin-overview.title",
  descriptionKey: "memory.admin-overview.description",
  permission: "memory.backend.commercialReadiness.read",
  resources: ["providerHealth","commercialReadiness"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminOverviewModule;
