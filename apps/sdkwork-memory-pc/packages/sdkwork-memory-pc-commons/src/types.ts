export type MemoryPcSurface = "app-console" | "backend-admin";

export type MemoryPcResourceKey =
  | "auditLogs"
  | "bindings"
  | "candidates"
  | "capabilities"
  | "capabilityBindings"
  | "commercialReadiness"
  | "consolidationJobs"
  | "contextPacks"
  | "edges"
  | "entities"
  | "evalRuns"
  | "events"
  | "exportJobs"
  | "extractionJobs"
  | "feedback"
  | "forgetRequests"
  | "habits"
  | "implementationProfiles"
  | "indexes"
  | "learningSettings"
  | "memories"
  | "migrationJobs"
  | "policies"
  | "policyAssignments"
  | "providerBindings"
  | "providerHealth"
  | "retentionJobs"
  | "retrievalProfiles"
  | "retrievals"
  | "retrievalTraces"
  | "spaces"
  | "subjects";

export type MemoryMessageCatalog = Readonly<Record<string, string>>;

export interface MemoryPcModuleDefinition {
  descriptionKey: string;
  id: string;
  messages: Readonly<Record<string, MemoryMessageCatalog>>;
  permission: string;
  resources: readonly MemoryPcResourceKey[];
  route: string;
  surface: MemoryPcSurface;
  titleKey: string;
}

export interface MemoryPageInfo {
  cursor?: string;
  hasNext?: boolean;
  mode: "cursor" | "offset";
  nextCursor?: string;
  page?: number;
  pageSize?: number;
  total?: number;
}

export interface MemoryPageResult {
  items: readonly Record<string, unknown>[];
  pageInfo: MemoryPageInfo;
}

export interface MemoryListQuery {
  cursor?: string;
  pageSize: number;
  q?: string;
  spaceId?: string;
}

export interface MemoryResourceDataSource {
  actions?: readonly MemoryResourceAction[];
  kind: "list" | "retrieve";
  load(query: MemoryListQuery, signal?: AbortSignal): Promise<MemoryPageResult>;
}

export interface MemoryResourceActionContext {
  auditReason?: string;
  body: Record<string, unknown>;
  idempotencyKey?: string;
  selectedItem?: Record<string, unknown>;
}

export interface MemoryResourceAction {
  auditReasonField?: string;
  bodyTemplate: Record<string, unknown>;
  dangerous?: boolean;
  execute(context: MemoryResourceActionContext): Promise<unknown>;
  id: string;
  label: string;
  requireAuditReason?: boolean;
  requireIdempotencyKey?: boolean;
  requiresSelection?: boolean;
}

export type MemoryResourceRegistry = Partial<Record<MemoryPcResourceKey, MemoryResourceDataSource>>;
