import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { MemoryAuditLogList, MemoryCandidate, MemoryCandidateList, MemoryEvalRun, MemoryEvalRunList, MemoryEvalRunRequest, MemoryEvent, MemoryEventList, MemoryExtractionRequest, MemoryImplementationProfile, MemoryImplementationProfileList, MemoryImplementationProfileRequest, MemoryIndex, MemoryIndexList, MemoryIndexRequest, MemoryLearningJob, MemoryMigrationJobRequest, MemoryProviderBinding, MemoryProviderBindingList, MemoryProviderBindingRequest, MemoryProviderHealth, MemoryRecord, MemoryRecordList, MemoryRecordRequest, MemoryRetentionJobRequest, MemoryRetrievalProfile, MemoryRetrievalProfileList, MemoryRetrievalProfileRequest, MemoryRetrievalTrace, MemoryRetrievalTraceList, MemoryReviewRequest, MemorySpace, MemorySpaceList, MemorySpaceRequest } from '../types';


export interface MemoryMigrationJobsCreateParams {
  idempotencyKey?: string;
}

export class MemoryMigrationJobsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryMigrationJobRequest, params?: MemoryMigrationJobsCreateParams): Promise<MemoryLearningJob> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryLearningJob>(backendApiPath(`/memory/migration_jobs`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(migrationJobId: string): Promise<MemoryLearningJob> {
    return this.client.get<MemoryLearningJob>(backendApiPath(`/memory/migration_jobs/${serializePathParameter(migrationJobId, { name: 'migrationJobId', style: 'simple', explode: false })}`));
  }
}

export interface MemoryRetentionJobsCreateParams {
  idempotencyKey?: string;
}

export class MemoryRetentionJobsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryRetentionJobRequest, params?: MemoryRetentionJobsCreateParams): Promise<MemoryLearningJob> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryLearningJob>(backendApiPath(`/memory/retention_jobs`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface MemoryAuditLogsListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export class MemoryAuditLogsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemoryAuditLogsListParams): Promise<MemoryAuditLogList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryAuditLogList>(appendQueryString(backendApiPath(`/memory/audit_logs`), query));
  }
}

export interface MemoryRetrievalTracesListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export class MemoryRetrievalTracesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemoryRetrievalTracesListParams): Promise<MemoryRetrievalTraceList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryRetrievalTraceList>(appendQueryString(backendApiPath(`/memory/retrieval_traces`), query));
  }

async retrieve(traceId: string): Promise<MemoryRetrievalTrace> {
    return this.client.get<MemoryRetrievalTrace>(backendApiPath(`/memory/retrieval_traces/${serializePathParameter(traceId, { name: 'traceId', style: 'simple', explode: false })}`));
  }
}

export interface MemoryEvalRunsListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export interface MemoryEvalRunsCreateParams {
  idempotencyKey?: string;
}

export class MemoryEvalRunsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemoryEvalRunsListParams): Promise<MemoryEvalRunList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryEvalRunList>(appendQueryString(backendApiPath(`/memory/eval_runs`), query));
  }

async create(body: MemoryEvalRunRequest, params?: MemoryEvalRunsCreateParams): Promise<MemoryEvalRun> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryEvalRun>(backendApiPath(`/memory/eval_runs`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(evalRunId: string): Promise<MemoryEvalRun> {
    return this.client.get<MemoryEvalRun>(backendApiPath(`/memory/eval_runs/${serializePathParameter(evalRunId, { name: 'evalRunId', style: 'simple', explode: false })}`));
  }
}

export class MemoryProviderHealthApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<MemoryProviderHealth> {
    return this.client.get<MemoryProviderHealth>(backendApiPath(`/memory/provider_health`));
  }
}

export interface MemoryProviderBindingsListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export interface MemoryProviderBindingsCreateParams {
  idempotencyKey?: string;
}

export class MemoryProviderBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemoryProviderBindingsListParams): Promise<MemoryProviderBindingList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryProviderBindingList>(appendQueryString(backendApiPath(`/memory/provider_bindings`), query));
  }

async create(body: MemoryProviderBindingRequest, params?: MemoryProviderBindingsCreateParams): Promise<MemoryProviderBinding> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryProviderBinding>(backendApiPath(`/memory/provider_bindings`), body, undefined, requestHeaders, 'application/json');
  }

async update(providerBindingId: string, body: MemoryProviderBindingRequest): Promise<MemoryProviderBinding> {
    return this.client.patch<MemoryProviderBinding>(backendApiPath(`/memory/provider_bindings/${serializePathParameter(providerBindingId, { name: 'providerBindingId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export interface MemoryImplementationProfilesListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export interface MemoryImplementationProfilesCreateParams {
  idempotencyKey?: string;
}

export class MemoryImplementationProfilesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemoryImplementationProfilesListParams): Promise<MemoryImplementationProfileList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryImplementationProfileList>(appendQueryString(backendApiPath(`/memory/implementation_profiles`), query));
  }

async create(body: MemoryImplementationProfileRequest, params?: MemoryImplementationProfilesCreateParams): Promise<MemoryImplementationProfile> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryImplementationProfile>(backendApiPath(`/memory/implementation_profiles`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(implementationProfileId: string): Promise<MemoryImplementationProfile> {
    return this.client.get<MemoryImplementationProfile>(backendApiPath(`/memory/implementation_profiles/${serializePathParameter(implementationProfileId, { name: 'implementationProfileId', style: 'simple', explode: false })}`));
  }

async update(implementationProfileId: string, body: MemoryImplementationProfileRequest): Promise<MemoryImplementationProfile> {
    return this.client.patch<MemoryImplementationProfile>(backendApiPath(`/memory/implementation_profiles/${serializePathParameter(implementationProfileId, { name: 'implementationProfileId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export interface MemoryRetrievalProfilesListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export interface MemoryRetrievalProfilesCreateParams {
  idempotencyKey?: string;
}

export class MemoryRetrievalProfilesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemoryRetrievalProfilesListParams): Promise<MemoryRetrievalProfileList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryRetrievalProfileList>(appendQueryString(backendApiPath(`/memory/retrieval_profiles`), query));
  }

async create(body: MemoryRetrievalProfileRequest, params?: MemoryRetrievalProfilesCreateParams): Promise<MemoryRetrievalProfile> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryRetrievalProfile>(backendApiPath(`/memory/retrieval_profiles`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(profileId: string): Promise<MemoryRetrievalProfile> {
    return this.client.get<MemoryRetrievalProfile>(backendApiPath(`/memory/retrieval_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`));
  }

async update(profileId: string, body: MemoryRetrievalProfileRequest): Promise<MemoryRetrievalProfile> {
    return this.client.patch<MemoryRetrievalProfile>(backendApiPath(`/memory/retrieval_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export interface MemoryIndexesListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export interface MemoryIndexesCreateParams {
  idempotencyKey?: string;
}

export interface MemoryIndexesRebuildParams {
  idempotencyKey?: string;
}

export class MemoryIndexesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemoryIndexesListParams): Promise<MemoryIndexList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryIndexList>(appendQueryString(backendApiPath(`/memory/indexes`), query));
  }

async create(body: MemoryIndexRequest, params?: MemoryIndexesCreateParams): Promise<MemoryIndex> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryIndex>(backendApiPath(`/memory/indexes`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(indexId: string): Promise<MemoryIndex> {
    return this.client.get<MemoryIndex>(backendApiPath(`/memory/indexes/${serializePathParameter(indexId, { name: 'indexId', style: 'simple', explode: false })}`));
  }

async update(indexId: string, body: MemoryIndexRequest): Promise<MemoryIndex> {
    return this.client.patch<MemoryIndex>(backendApiPath(`/memory/indexes/${serializePathParameter(indexId, { name: 'indexId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

async rebuild(indexId: string, body: MemoryReviewRequest, params?: MemoryIndexesRebuildParams): Promise<MemoryLearningJob> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryLearningJob>(backendApiPath(`/memory/indexes/${serializePathParameter(indexId, { name: 'indexId', style: 'simple', explode: false })}/rebuild`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface MemoryConsolidationJobsCreateParams {
  idempotencyKey?: string;
}

export class MemoryConsolidationJobsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryExtractionRequest, params?: MemoryConsolidationJobsCreateParams): Promise<MemoryLearningJob> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryLearningJob>(backendApiPath(`/memory/consolidation_jobs`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface MemoryExtractionJobsCreateParams {
  idempotencyKey?: string;
}

export class MemoryExtractionJobsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryExtractionRequest, params?: MemoryExtractionJobsCreateParams): Promise<MemoryLearningJob> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryLearningJob>(backendApiPath(`/memory/extraction_jobs`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(jobId: string): Promise<MemoryLearningJob> {
    return this.client.get<MemoryLearningJob>(backendApiPath(`/memory/extraction_jobs/${serializePathParameter(jobId, { name: 'jobId', style: 'simple', explode: false })}`));
  }
}

export interface MemoryCandidatesListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export interface MemoryCandidatesApproveParams {
  idempotencyKey?: string;
}

export interface MemoryCandidatesRejectParams {
  idempotencyKey?: string;
}

export class MemoryCandidatesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemoryCandidatesListParams): Promise<MemoryCandidateList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryCandidateList>(appendQueryString(backendApiPath(`/memory/candidates`), query));
  }

async approve(candidateId: string, body: MemoryReviewRequest, params?: MemoryCandidatesApproveParams): Promise<MemoryCandidate> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryCandidate>(backendApiPath(`/memory/candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/approve`), body, undefined, requestHeaders, 'application/json');
  }

async reject(candidateId: string, body: MemoryReviewRequest, params?: MemoryCandidatesRejectParams): Promise<MemoryCandidate> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryCandidate>(backendApiPath(`/memory/candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/reject`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface MemoryEventsListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export class MemoryEventsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemoryEventsListParams): Promise<MemoryEventList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryEventList>(appendQueryString(backendApiPath(`/memory/events`), query));
  }

async retrieve(eventId: string): Promise<MemoryEvent> {
    return this.client.get<MemoryEvent>(backendApiPath(`/memory/events/${serializePathParameter(eventId, { name: 'eventId', style: 'simple', explode: false })}`));
  }
}

export interface MemorySpacesListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export class MemorySpacesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemorySpacesListParams): Promise<MemorySpaceList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemorySpaceList>(appendQueryString(backendApiPath(`/memory/spaces`), query));
  }

async retrieve(spaceId: string): Promise<MemorySpace> {
    return this.client.get<MemorySpace>(backendApiPath(`/memory/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`));
  }

async update(spaceId: string, body: MemorySpaceRequest): Promise<MemorySpace> {
    return this.client.patch<MemorySpace>(backendApiPath(`/memory/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export interface MemoryListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export interface MemorySupersedeParams {
  idempotencyKey?: string;
}

export class MemoryApi {
  private client: HttpClient;
  public readonly spaces: MemorySpacesApi;
  public readonly events: MemoryEventsApi;
  public readonly candidates: MemoryCandidatesApi;
  public readonly extractionJobs: MemoryExtractionJobsApi;
  public readonly consolidationJobs: MemoryConsolidationJobsApi;
  public readonly indexes: MemoryIndexesApi;
  public readonly retrievalProfiles: MemoryRetrievalProfilesApi;
  public readonly implementationProfiles: MemoryImplementationProfilesApi;
  public readonly providerBindings: MemoryProviderBindingsApi;
  public readonly providerHealth: MemoryProviderHealthApi;
  public readonly evalRuns: MemoryEvalRunsApi;
  public readonly retrievalTraces: MemoryRetrievalTracesApi;
  public readonly auditLogs: MemoryAuditLogsApi;
  public readonly retentionJobs: MemoryRetentionJobsApi;
  public readonly migrationJobs: MemoryMigrationJobsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.spaces = new MemorySpacesApi(client);
    this.events = new MemoryEventsApi(client);
    this.candidates = new MemoryCandidatesApi(client);
    this.extractionJobs = new MemoryExtractionJobsApi(client);
    this.consolidationJobs = new MemoryConsolidationJobsApi(client);
    this.indexes = new MemoryIndexesApi(client);
    this.retrievalProfiles = new MemoryRetrievalProfilesApi(client);
    this.implementationProfiles = new MemoryImplementationProfilesApi(client);
    this.providerBindings = new MemoryProviderBindingsApi(client);
    this.providerHealth = new MemoryProviderHealthApi(client);
    this.evalRuns = new MemoryEvalRunsApi(client);
    this.retrievalTraces = new MemoryRetrievalTracesApi(client);
    this.auditLogs = new MemoryAuditLogsApi(client);
    this.retentionJobs = new MemoryRetentionJobsApi(client);
    this.migrationJobs = new MemoryMigrationJobsApi(client);
  }


async list(params?: MemoryListParams): Promise<MemoryRecordList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryRecordList>(appendQueryString(backendApiPath(`/memory/memories`), query));
  }

async retrieve(memoryId: string): Promise<MemoryRecord> {
    return this.client.get<MemoryRecord>(backendApiPath(`/memory/memories/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}`));
  }

async update(memoryId: string, body: MemoryRecordRequest): Promise<MemoryRecord> {
    return this.client.patch<MemoryRecord>(backendApiPath(`/memory/memories/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

async supersede(memoryId: string, body: MemoryRecordRequest, params?: MemorySupersedeParams): Promise<MemoryRecord> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryRecord>(backendApiPath(`/memory/memories/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}/supersede`), body, undefined, requestHeaders, 'application/json');
  }
}

export function createMemoryApi(client: HttpClient): MemoryApi {
  return new MemoryApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}

interface PathParameterSpec {
  name: string;
  style: string;
  explode: boolean;
}

function serializePathParameter(value: unknown, spec: PathParameterSpec): string {
  if (value === undefined || value === null) {
    return '';
  }

  const style = spec.style || 'simple';
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === 'object') {
    return serializePathObject(spec.name, value as Record<string, unknown>, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}

function serializePathArray(name: string, values: unknown[], style: string, explode: boolean): string {
  const serialized = values
    .filter((item) => item !== undefined && item !== null)
    .map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === 'matrix') {
    return explode
      ? serialized.map((item) => `;${name}=${item}`).join('')
      : `;${name}=${serialized.join(',')}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? '.' : ',');
}

function serializePathObject(name: string, value: Record<string, unknown>, style: string, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === 'matrix') {
    return explode
      ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
      : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
  }
  const serialized = explode
    ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
    : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
  return pathPrefix(name, style, true) + serialized;
}

function pathPrefix(name: string, style: string, _objectValue: boolean): string {
  if (style === 'label') return '.';
  if (style === 'matrix') return `;${name}`;
  return '';
}

function encodePathValue(value: string): string {
  return encodeURIComponent(value);
}

function serializePathPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}
interface QueryParameterSpec {
  name: string;
  value: unknown;
  style: string;
  explode: boolean;
  allowReserved: boolean;
  contentType?: string;
}

function buildQueryString(parameters: QueryParameterSpec[]): string {
  const pairs: string[] = [];
  for (const parameter of parameters) {
    appendSerializedParameter(pairs, parameter);
  }
  return pairs.join('&');
}

function appendSerializedParameter(pairs: string[], parameter: QueryParameterSpec): void {
  if (parameter.value === undefined || parameter.value === null) {
    return;
  }

  if (parameter.contentType) {
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(JSON.stringify(parameter.value), parameter.allowReserved)}`);
    return;
  }

  const style = parameter.style || 'form';
  if (style === 'deepObject') {
    appendDeepObjectParameter(pairs, parameter.name, parameter.value, parameter.allowReserved);
    return;
  }

  if (Array.isArray(parameter.value)) {
    appendArrayParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
    return;
  }

  if (typeof parameter.value === 'object') {
    appendObjectParameter(pairs, parameter.name, parameter.value as Record<string, unknown>, style, parameter.explode, parameter.allowReserved);
    return;
  }

  pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}

function appendArrayParameter(
  pairs: string[],
  name: string,
  value: unknown[],
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const values = value
    .filter((item) => item !== undefined && item !== null)
    .map((item) => serializePrimitive(item));
  if (values.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const item of values) {
      pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(item, allowReserved)}`);
    }
    return;
  }

  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(values.join(','), allowReserved)}`);
}

function appendObjectParameter(
  pairs: string[],
  name: string,
  value: Record<string, unknown>,
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const [key, entryValue] of entries) {
      pairs.push(`${encodeQueryComponent(key)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
    return;
  }

  const serialized = entries.flatMap(([key, entryValue]) => [key, serializePrimitive(entryValue)]).join(',');
  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serialized, allowReserved)}`);
}

function appendDeepObjectParameter(
  pairs: string[],
  name: string,
  value: unknown,
  allowReserved: boolean,
): void {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
    return;
  }

  for (const [key, entryValue] of Object.entries(value as Record<string, unknown>)) {
    if (entryValue === undefined || entryValue === null) {
      continue;
    }
    pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
  }
}

function serializePrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}

function encodeQueryComponent(value: string): string {
  return encodeURIComponent(value);
}

function encodeQueryValue(value: string, allowReserved: boolean): string {
  const encoded = encodeURIComponent(value);
  if (!allowReserved) {
    return encoded;
  }
  return encoded.replace(/%3A/gi, ':')
    .replace(/%2F/gi, '/')
    .replace(/%3F/gi, '?')
    .replace(/%23/gi, '#')
    .replace(/%5B/gi, '[')
    .replace(/%5D/gi, ']')
    .replace(/%40/gi, '@')
    .replace(/%21/gi, '!')
    .replace(/%24/gi, '$')
    .replace(/%26/gi, '&')
    .replace(/%27/gi, "'")
    .replace(/%28/gi, '(')
    .replace(/%29/gi, ')')
    .replace(/%2A/gi, '*')
    .replace(/%2B/gi, '+')
    .replace(/%2C/gi, ',')
    .replace(/%3B/gi, ';')
    .replace(/%3D/gi, '=');
}
function buildRequestHeaders(
  headers: Record<string, HeaderParameterSpec | undefined>,
  cookies: Record<string, HeaderParameterSpec | undefined> = {},
): Record<string, string> | undefined {
  const requestHeaders: Record<string, string> = {};

  for (const [name, parameter] of Object.entries(headers)) {
    const serialized = serializeParameterValue(parameter);
    if (serialized !== undefined) {
      requestHeaders[name] = serialized;
    }
  }

  const cookieHeader = buildCookieHeader(cookies);
  if (cookieHeader) {
    requestHeaders.Cookie = requestHeaders.Cookie
      ? `${requestHeaders.Cookie}; ${cookieHeader}`
      : cookieHeader;
  }

  return Object.keys(requestHeaders).length > 0 ? requestHeaders : undefined;
}

interface HeaderParameterSpec {
  value: unknown;
  style: string;
  explode: boolean;
  contentType?: string;
}

function buildCookieHeader(cookies: Record<string, HeaderParameterSpec | undefined>): string | undefined {
  const pairs: string[] = [];
  for (const [name, parameter] of Object.entries(cookies)) {
    const serialized = serializeParameterValue(parameter);
    if (serialized !== undefined) {
      pairs.push(`${encodeURIComponent(name)}=${encodeURIComponent(serialized)}`);
    }
  }
  return pairs.length > 0 ? pairs.join('; ') : undefined;
}

function serializeParameterValue(parameter: HeaderParameterSpec | undefined): string | undefined {
  const value = parameter?.value;
  if (value === undefined || value === null) {
    return undefined;
  }
  if (parameter?.contentType) {
    return JSON.stringify(value);
  }
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (Array.isArray(value)) {
    return value.map((item) => serializeHeaderPrimitive(item)).join(',');
  }
  if (typeof value === 'object' && value !== null) {
    return serializeHeaderObject(value as Record<string, unknown>, parameter?.explode === true);
  }
  return serializeHeaderPrimitive(value);
}

function serializeHeaderObject(value: Record<string, unknown>, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (explode) {
    return entries.map(([key, entryValue]) => `${key}=${serializeHeaderPrimitive(entryValue)}`).join(',');
  }
  return entries.flatMap(([key, entryValue]) => [key, serializeHeaderPrimitive(entryValue)]).join(',');
}

function serializeHeaderPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  return String(value);
}
