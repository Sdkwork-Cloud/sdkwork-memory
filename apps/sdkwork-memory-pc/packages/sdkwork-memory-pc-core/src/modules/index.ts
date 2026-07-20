import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";

export function assertUniqueMemoryModules(modules: readonly MemoryPcModuleDefinition[]): readonly MemoryPcModuleDefinition[] {
  const ids = new Set<string>();
  const routes = new Set<string>();
  for (const module of modules) {
    if (ids.has(module.id)) throw new Error(`Duplicate Memory module id: ${module.id}`);
    if (routes.has(`${module.surface}:${module.route}`)) throw new Error(`Duplicate Memory module route: ${module.surface}/${module.route}`);
    ids.add(module.id);
    routes.add(`${module.surface}:${module.route}`);
  }
  return modules;
}
