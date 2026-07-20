import { MemoryAdminSdkProvider, createMemoryAdminResourceRegistry, createMemoryAdminSdkClient } from "@sdkwork/memory-pc-admin-core";
import { MemoryAdminShell } from "@sdkwork/memory-pc-admin-shell";
import type { MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import type { AuthTokenManager } from "@sdkwork/sdk-common";
import { useMemo } from "react";

export interface MemoryAdminSurfaceProps {
  backendApiBaseUrl: string;
  modules: readonly MemoryPcModuleDefinition[];
  onSignOut(): void;
  permissionScope: readonly string[];
  tokenManager: AuthTokenManager;
  userLabel?: string;
}

export function MemoryAdminSurface({ backendApiBaseUrl, modules, onSignOut, permissionScope, tokenManager, userLabel }: MemoryAdminSurfaceProps) {
  const client = useMemo(() => createMemoryAdminSdkClient(backendApiBaseUrl, tokenManager), [backendApiBaseUrl, tokenManager]);
  const registry = useMemo(() => createMemoryAdminResourceRegistry(client), [client]);
  return <MemoryAdminSdkProvider client={client}><MemoryAdminShell modules={modules} permissionScope={permissionScope} registry={registry} userLabel={userLabel} onSignOut={onSignOut} /></MemoryAdminSdkProvider>;
}
