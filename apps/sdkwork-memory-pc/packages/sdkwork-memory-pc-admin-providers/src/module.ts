import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export const adminProvidersModule = {
  id: "admin-providers",
  surface: "backend-admin",
  route: "providers",
  titleKey: "memory.admin-providers.title",
  descriptionKey: "memory.admin-providers.description",
  permission: "memory.backend.providerBindings.read",
  resources: ["implementationProfiles","providerBindings","providerHealth"],
} as const satisfies MemoryPcModuleDefinition;

export const memoryModule = adminProvidersModule;
