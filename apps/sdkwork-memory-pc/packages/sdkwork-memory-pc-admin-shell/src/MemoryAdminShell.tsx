import { MemorySurfaceWorkspace, type MemoryPcModuleDefinition, type MemoryResourceRegistry } from "@sdkwork/memory-pc-commons";

export interface MemoryAdminShellProps {
  modules: readonly MemoryPcModuleDefinition[];
  onSignOut?(): void;
  permissionScope: readonly string[];
  registry: MemoryResourceRegistry;
  userLabel?: string;
}

export function MemoryAdminShell(props: MemoryAdminShellProps) {
  return <MemorySurfaceWorkspace {...props} surface="backend-admin" />;
}
