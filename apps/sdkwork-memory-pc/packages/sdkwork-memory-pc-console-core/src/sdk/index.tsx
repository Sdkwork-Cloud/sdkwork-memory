import { normalizeMemoryItem, normalizeMemoryPage, type MemoryListQuery, type MemoryResourceDataSource, type MemoryResourceRegistry } from "@sdkwork/memory-pc-commons";
import type { SdkworkAppClient } from "@sdkwork/memory-app-sdk";
import { createContext, useContext, type ReactNode } from "react";

export type MemoryConsoleSdkClient = SdkworkAppClient;

const MemoryConsoleSdkContext = createContext<MemoryConsoleSdkClient | null>(null);

export function MemoryConsoleSdkProvider({ children, client }: { children: ReactNode; client: MemoryConsoleSdkClient }) {
  return <MemoryConsoleSdkContext.Provider value={client}>{children}</MemoryConsoleSdkContext.Provider>;
}

export function useMemoryConsoleSdk(): MemoryConsoleSdkClient {
  const client = useContext(MemoryConsoleSdkContext);
  if (!client) throw new Error("MemoryConsoleSdkProvider is required");
  return client;
}

export function createMemoryConsoleResourceRegistry(client: MemoryConsoleSdkClient): MemoryResourceRegistry {
  return {
    spaces: listSource((query) => client.memory.spaces.list(toListParams(query))),
    memories: listSource((query) => query.spaceId
      ? client.memory.list({ ...toListParams(query), spaceId: query.spaceId })
      : Promise.resolve({ items: [], pageInfo: { mode: "cursor", hasNext: false } })),
    candidates: listSource((query) => client.memory.candidates.list(toListParams(query))),
    habits: listSource((query) => client.memory.habits.list(toListParams(query))),
    learningSettings: itemSource(() => client.memory.learningSettings.retrieve()),
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
