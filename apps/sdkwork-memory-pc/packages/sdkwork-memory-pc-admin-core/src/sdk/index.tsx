import { normalizeMemoryItem, normalizeMemoryPage, type MemoryListQuery, type MemoryResourceDataSource, type MemoryResourceRegistry } from "@sdkwork/memory-pc-commons";
import { createClient, type SdkworkBackendClient } from "@sdkwork/memory-backend-sdk";
import type { AuthTokenManager } from "@sdkwork/sdk-common";
import { createContext, useContext, type ReactNode } from "react";

export type MemoryAdminSdkClient = SdkworkBackendClient;

const MemoryAdminSdkContext = createContext<MemoryAdminSdkClient | null>(null);

export function createMemoryAdminSdkClient(baseUrl: string, tokenManager: AuthTokenManager): MemoryAdminSdkClient {
  return createClient({ baseUrl, authMode: "dual-token", platform: "pc", tokenManager });
}

export function MemoryAdminSdkProvider({ children, client }: { children: ReactNode; client: MemoryAdminSdkClient }) {
  return <MemoryAdminSdkContext.Provider value={client}>{children}</MemoryAdminSdkContext.Provider>;
}

export function useMemoryAdminSdk(): MemoryAdminSdkClient {
  const client = useContext(MemoryAdminSdkContext);
  if (!client) throw new Error("MemoryAdminSdkProvider is required");
  return client;
}

export function createMemoryAdminResourceRegistry(client: MemoryAdminSdkClient): MemoryResourceRegistry {
  return {
    spaces: listSource((query) => client.memory.spaces.list(toListParams(query))),
    memories: listSource((query) => client.memory.list(toListParams(query))),
    events: listSource((query) => client.memory.events.list(toListParams(query))),
    candidates: listSource((query) => client.memory.candidates.list(toListParams(query))),
    indexes: listSource((query) => client.memory.indexes.list(toListParams(query))),
    retrievalProfiles: listSource((query) => client.memory.retrievalProfiles.list(toListParams(query))),
    retrievalTraces: listSource((query) => client.memory.retrievalTraces.list(toListParams(query))),
    implementationProfiles: listSource((query) => client.memory.implementationProfiles.list(toListParams(query))),
    providerBindings: listSource((query) => client.memory.providerBindings.list(toListParams(query))),
    providerHealth: itemSource(() => client.memory.providerHealth.retrieve()),
    evalRuns: listSource((query) => client.memory.evalRuns.list(toListParams(query))),
    auditLogs: listSource((query) => client.memory.auditLogs.list(toListParams(query))),
  };
}

function listSource(load: (query: MemoryListQuery) => Promise<unknown>): MemoryResourceDataSource {
  return { kind: "list", async load(query) { return normalizeMemoryPage(await load(query)); } };
}

function itemSource(load: () => Promise<unknown>): MemoryResourceDataSource {
  return { kind: "retrieve", async load() { return normalizeMemoryItem(await load()); } };
}

function toListParams(query: MemoryListQuery) {
  return { q: query.q, cursor: query.cursor, pageSize: query.pageSize };
}
