import { normalizeMemoryItem, normalizeMemoryPage, type MemoryListQuery, type MemoryResourceAction, type MemoryResourceActionContext, type MemoryResourceDataSource, type MemoryResourceRegistry } from "@sdkwork/memory-pc-commons";
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
  const idempotency = (context: MemoryResourceActionContext) => ({ idempotencyKey: context.idempotencyKey });
  return {
    spaces: withActions(listSource((query) => client.memory.spaces.list(toListParams(query))), [
      action("create", "Create space", { ownerSubjectType: "user", ownerSubjectId: "", spaceType: "personal", displayName: "" }, (context) => client.memory.spaces.create(context.body as unknown as Parameters<typeof client.memory.spaces.create>[0], idempotency(context)), { idempotent: true }),
      action("update", "Update space", { ownerSubjectType: "user", ownerSubjectId: "", spaceType: "personal", displayName: "", lifecycleStatus: "active", version: "" }, (context) => client.memory.spaces.update(selectedId(context, "spaceId"), context.body as unknown as Parameters<typeof client.memory.spaces.update>[1]), { selection: true }),
    ]),
    memories: withActions(listSource((query) => query.spaceId
      ? client.memory.list({ ...toListParams(query), spaceId: query.spaceId })
      : Promise.resolve({ items: [], pageInfo: { mode: "cursor", hasNext: false } })), [
      action("create", "Create memory", { spaceId: "", scope: "user", memoryType: "semantic", canonicalText: "", sensitivityLevel: "internal" }, (context) => client.memory.create(context.body as unknown as Parameters<typeof client.memory.create>[0], idempotency(context)), { idempotent: true }),
      action("update", "Update memory", { spaceId: "", canonicalText: "", subject: "", summaryText: "", metadata: {} }, (context) => { const { spaceId, ...patch } = context.body; return client.memory.update(selectedId(context, "memoryId"), patch as unknown as Parameters<typeof client.memory.update>[1], { spaceId: String(spaceId ?? "") }); }, { selection: true }),
      action("delete", "Delete memory", { spaceId: "" }, (context) => client.memory.delete(selectedId(context, "memoryId"), { spaceId: String(context.body.spaceId ?? "") }), { dangerous: true, selection: true }),
    ]),
    candidates: withActions(listSource((query) => client.memory.candidates.list(toListParams(query))), [
      action("approve", "Approve candidate", { reason: "" }, (context) => client.memory.candidates.approve(selectedId(context, "candidateId"), context.body as unknown as Parameters<typeof client.memory.candidates.approve>[1], idempotency(context)), { idempotent: true, selection: true }),
      action("reject", "Reject candidate", { reason: "" }, (context) => client.memory.candidates.reject(selectedId(context, "candidateId"), context.body as unknown as Parameters<typeof client.memory.candidates.reject>[1], idempotency(context)), { dangerous: true, idempotent: true, reason: true, selection: true }),
    ]),
    habits: withActions(listSource((query) => client.memory.habits.list(toListParams(query))), [
      action("update", "Update habit", { description: "", confidence: 0.5, version: "" }, (context) => client.memory.habits.update(selectedId(context, "habitId"), context.body as unknown as Parameters<typeof client.memory.habits.update>[1]), { selection: true }),
      action("confirm", "Confirm habit", { reason: "" }, (context) => client.memory.habits.confirm(selectedId(context, "habitId"), context.body as unknown as Parameters<typeof client.memory.habits.confirm>[1], idempotency(context)), { idempotent: true, selection: true }),
      action("reject", "Reject habit", { reason: "" }, (context) => client.memory.habits.reject(selectedId(context, "habitId"), context.body as unknown as Parameters<typeof client.memory.habits.reject>[1], idempotency(context)), { dangerous: true, idempotent: true, reason: true, selection: true }),
    ]),
    learningSettings: withActions(itemSource(() => client.memory.learningSettings.retrieve()), [
      action("update", "Update settings", { autoExtractEnabled: true, autoApproveThreshold: 0.9, habitLearningEnabled: true }, (context) => client.memory.learningSettings.update(context.body as unknown as Parameters<typeof client.memory.learningSettings.update>[0])),
    ]),
    retrievals: actionSource([
      action("create", "Run retrieval", { query: "", spaceIds: [], topK: 10, contextBudgetTokens: 2048, includeTrace: true }, (context) => client.memory.retrievals.create(context.body as unknown as Parameters<typeof client.memory.retrievals.create>[0], idempotency(context)), { idempotent: true }),
    ]),
    contextPacks: actionSource([
      action("create", "Create context pack", { query: "", spaceIds: [], contextBudgetTokens: 2048, includeCitations: true }, (context) => client.memory.contextPacks.create(context.body as unknown as Parameters<typeof client.memory.contextPacks.create>[0], idempotency(context)), { idempotent: true }),
    ]),
    feedback: actionSource([
      action("create", "Submit feedback", { targetType: "retrieval", targetId: "", feedbackType: "relevance", score: 1, comment: "" }, (context) => client.memory.feedback.create(context.body as unknown as Parameters<typeof client.memory.feedback.create>[0], idempotency(context)), { idempotent: true }),
    ]),
    entities: withActions(listSource((query) => client.memory.entities.list({ ...toListParams(query), spaceId: query.spaceId })), [
      action("create", "Create entity", { spaceId: "", entityType: "person", canonicalName: "", sensitivityLevel: "internal" }, (context) => client.memory.entities.create(context.body as unknown as Parameters<typeof client.memory.entities.create>[0], idempotency(context)), { idempotent: true }),
      action("update", "Update entity", { canonicalName: "", status: "active" }, (context) => client.memory.entities.update(selectedId(context, "entityId"), context.body as unknown as Parameters<typeof client.memory.entities.update>[1]), { selection: true }),
    ]),
    policyAssignments: withActions(listSource((query) => client.memory.policyAssignments.list(toListParams(query))), [
      action("create", "Assign policy", { policyId: "", targetType: "space", targetId: "", priority: 0, inheritanceMode: "inherit" }, (context) => client.memory.policyAssignments.create(context.body as unknown as Parameters<typeof client.memory.policyAssignments.create>[0], idempotency(context)), { idempotent: true }),
      action("update", "Update assignment", { priority: 0, inheritanceMode: "inherit", status: "active" }, (context) => client.memory.policyAssignments.update(selectedId(context, "policyAssignmentId"), context.body as unknown as Parameters<typeof client.memory.policyAssignments.update>[1]), { selection: true }),
    ]),
    forgetRequests: withActions(listSource((query) => client.memory.forgetRequests.list(toJobListParams(query))), [
      action("create", "Create forget request", { scope: "memory", memoryIds: [], reason: "" }, (context) => client.memory.forgetRequests.create(context.body as unknown as Parameters<typeof client.memory.forgetRequests.create>[0], idempotency(context)), { dangerous: true, idempotent: true, reason: true }),
    ]),
    exportJobs: withActions(listSource((query) => client.memory.exportJobs.list(toJobListParams(query))), [
      action("create", "Create export", { spaceIds: [], format: "json", includeEvents: true }, (context) => client.memory.exportJobs.create(context.body as unknown as Parameters<typeof client.memory.exportJobs.create>[0], idempotency(context)), { idempotent: true }),
    ]),
  };
}

function action(id: string, label: string, bodyTemplate: Record<string, unknown>, execute: MemoryResourceAction["execute"], options: { dangerous?: boolean; idempotent?: boolean; reason?: boolean; selection?: boolean } = {}): MemoryResourceAction {
  return { id, label, bodyTemplate, execute, dangerous: options.dangerous, requireAuditReason: options.reason, auditReasonField: options.reason ? "reason" : undefined, requireIdempotencyKey: options.idempotent, requiresSelection: options.selection };
}

function withActions(source: MemoryResourceDataSource, actions: readonly MemoryResourceAction[]): MemoryResourceDataSource {
  return { ...source, actions };
}

function actionSource(actions: readonly MemoryResourceAction[]): MemoryResourceDataSource {
  return { actions, kind: "retrieve", async load() { return { items: [], pageInfo: { mode: "cursor", hasNext: false } }; } };
}

function selectedId(context: MemoryResourceActionContext, ...keys: string[]): string {
  for (const key of keys) {
    const value = context.selectedItem?.[key];
    if (typeof value === "string" || typeof value === "number") return String(value);
  }
  throw new Error("Selected resource id is unavailable");
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

function toJobListParams(query: MemoryListQuery) {
  return { cursor: query.cursor, pageSize: query.pageSize };
}
