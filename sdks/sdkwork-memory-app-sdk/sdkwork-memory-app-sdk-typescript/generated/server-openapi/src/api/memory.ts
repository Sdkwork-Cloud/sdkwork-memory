import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { MemoryCandidate, MemoryCandidateList, MemoryContextPack, MemoryContextPackRequest, MemoryEvent, MemoryEventRequest, MemoryExportJob, MemoryExportRequest, MemoryExtractionRequest, MemoryFeedback, MemoryFeedbackRequest, MemoryForgetJob, MemoryForgetRequest, MemoryHabit, MemoryHabitList, MemoryHabitRequest, MemoryLearningJob, MemoryLearningSettings, MemoryLearningSettingsRequest, MemoryRecord, MemoryRecordList, MemoryRecordRequest, MemoryRecordSourceList, MemoryRetrievalRequest, MemoryRetrievalResult, MemoryReviewRequest, MemorySpace, MemorySpaceList, MemorySpaceRequest } from '../types';


export class MemoryLearningSettingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<MemoryLearningSettings> {
    return this.client.get<MemoryLearningSettings>(appApiPath(`/memory/learning_settings`));
  }

async update(body: MemoryLearningSettingsRequest): Promise<MemoryLearningSettings> {
    return this.client.patch<MemoryLearningSettings>(appApiPath(`/memory/learning_settings`), body, undefined, undefined, 'application/json');
  }
}

export interface MemoryExportJobsCreateParams {
  idempotencyKey?: string;
}

export class MemoryExportJobsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryExportRequest, params?: MemoryExportJobsCreateParams): Promise<MemoryExportJob> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryExportJob>(appApiPath(`/memory/export_jobs`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(exportJobId: string): Promise<MemoryExportJob> {
    return this.client.get<MemoryExportJob>(appApiPath(`/memory/export_jobs/${serializePathParameter(exportJobId, { name: 'exportJobId', style: 'simple', explode: false })}`));
  }
}

export interface MemoryFeedbackCreateParams {
  idempotencyKey?: string;
}

export class MemoryFeedbackApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryFeedbackRequest, params?: MemoryFeedbackCreateParams): Promise<MemoryFeedback> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryFeedback>(appApiPath(`/memory/feedback`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface MemoryContextPacksCreateParams {
  idempotencyKey?: string;
}

export class MemoryContextPacksApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryContextPackRequest, params?: MemoryContextPacksCreateParams): Promise<MemoryContextPack> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryContextPack>(appApiPath(`/memory/context_packs`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(contextPackId: string): Promise<MemoryContextPack> {
    return this.client.get<MemoryContextPack>(appApiPath(`/memory/context_packs/${serializePathParameter(contextPackId, { name: 'contextPackId', style: 'simple', explode: false })}`));
  }
}

export interface MemoryRetrievalsCreateParams {
  idempotencyKey?: string;
}

export class MemoryRetrievalsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryRetrievalRequest, params?: MemoryRetrievalsCreateParams): Promise<MemoryRetrievalResult> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryRetrievalResult>(appApiPath(`/memory/retrievals`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(retrievalId: string): Promise<MemoryRetrievalResult> {
    return this.client.get<MemoryRetrievalResult>(appApiPath(`/memory/retrievals/${serializePathParameter(retrievalId, { name: 'retrievalId', style: 'simple', explode: false })}`));
  }
}

export interface MemoryHabitsListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
  stage?: string;
}

export interface MemoryHabitsConfirmParams {
  idempotencyKey?: string;
}

export interface MemoryHabitsRejectParams {
  idempotencyKey?: string;
}

export class MemoryHabitsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(params?: MemoryHabitsListParams): Promise<MemoryHabitList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'stage', value: params?.stage, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryHabitList>(appendQueryString(appApiPath(`/memory/habits`), query));
  }

async retrieve(habitId: string): Promise<MemoryHabit> {
    return this.client.get<MemoryHabit>(appApiPath(`/memory/habits/${serializePathParameter(habitId, { name: 'habitId', style: 'simple', explode: false })}`));
  }

async update(habitId: string, body: MemoryHabitRequest): Promise<MemoryHabit> {
    return this.client.patch<MemoryHabit>(appApiPath(`/memory/habits/${serializePathParameter(habitId, { name: 'habitId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

async confirm(habitId: string, body: MemoryReviewRequest, params?: MemoryHabitsConfirmParams): Promise<MemoryHabit> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryHabit>(appApiPath(`/memory/habits/${serializePathParameter(habitId, { name: 'habitId', style: 'simple', explode: false })}/confirm`), body, undefined, requestHeaders, 'application/json');
  }

async reject(habitId: string, body: MemoryReviewRequest, params?: MemoryHabitsRejectParams): Promise<MemoryHabit> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryHabit>(appApiPath(`/memory/habits/${serializePathParameter(habitId, { name: 'habitId', style: 'simple', explode: false })}/reject`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface MemoryCandidatesListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
  decisionState?: string;
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
      { name: 'decision_state', value: params?.decisionState, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryCandidateList>(appendQueryString(appApiPath(`/memory/candidates`), query));
  }

async retrieve(candidateId: string): Promise<MemoryCandidate> {
    return this.client.get<MemoryCandidate>(appApiPath(`/memory/candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}`));
  }

async approve(candidateId: string, body: MemoryReviewRequest, params?: MemoryCandidatesApproveParams): Promise<MemoryCandidate> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryCandidate>(appApiPath(`/memory/candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/approve`), body, undefined, requestHeaders, 'application/json');
  }

async reject(candidateId: string, body: MemoryReviewRequest, params?: MemoryCandidatesRejectParams): Promise<MemoryCandidate> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryCandidate>(appApiPath(`/memory/candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/reject`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface MemoryExtractionsCreateParams {
  idempotencyKey?: string;
}

export class MemoryExtractionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryExtractionRequest, params?: MemoryExtractionsCreateParams): Promise<MemoryLearningJob> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryLearningJob>(appApiPath(`/memory/extractions`), body, undefined, requestHeaders, 'application/json');
  }
}

export interface MemoryForgetRequestsCreateParams {
  idempotencyKey?: string;
}

export class MemoryForgetRequestsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryForgetRequest, params?: MemoryForgetRequestsCreateParams): Promise<MemoryForgetJob> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryForgetJob>(appApiPath(`/memory/forget_requests`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(forgetRequestId: string): Promise<MemoryForgetJob> {
    return this.client.get<MemoryForgetJob>(appApiPath(`/memory/forget_requests/${serializePathParameter(forgetRequestId, { name: 'forgetRequestId', style: 'simple', explode: false })}`));
  }
}

export interface MemorySourcesListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export class MemorySourcesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async list(memoryId: string, params?: MemorySourcesListParams): Promise<MemoryRecordSourceList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryRecordSourceList>(appendQueryString(appApiPath(`/memory/memories/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}/sources`), query));
  }
}

export interface MemoryEventsCreateParams {
  idempotencyKey?: string;
}

export class MemoryEventsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async create(body: MemoryEventRequest, params?: MemoryEventsCreateParams): Promise<MemoryEvent> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryEvent>(appApiPath(`/memory/events`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(eventId: string): Promise<MemoryEvent> {
    return this.client.get<MemoryEvent>(appApiPath(`/memory/events/${serializePathParameter(eventId, { name: 'eventId', style: 'simple', explode: false })}`));
  }
}

export interface MemorySpacesListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
}

export interface MemorySpacesCreateParams {
  idempotencyKey?: string;
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
    return this.client.get<MemorySpaceList>(appendQueryString(appApiPath(`/memory/spaces`), query));
  }

async create(body: MemorySpaceRequest, params?: MemorySpacesCreateParams): Promise<MemorySpace> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemorySpace>(appApiPath(`/memory/spaces`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(spaceId: string): Promise<MemorySpace> {
    return this.client.get<MemorySpace>(appApiPath(`/memory/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`));
  }

async update(spaceId: string, body: MemorySpaceRequest): Promise<MemorySpace> {
    return this.client.patch<MemorySpace>(appApiPath(`/memory/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export interface MemoryListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
  spaceId?: string;
  memoryType?: string;
}

export interface MemoryCreateParams {
  idempotencyKey?: string;
}

export class MemoryApi {
  private client: HttpClient;
  public readonly spaces: MemorySpacesApi;
  public readonly events: MemoryEventsApi;
  public readonly sources: MemorySourcesApi;
  public readonly forgetRequests: MemoryForgetRequestsApi;
  public readonly extractions: MemoryExtractionsApi;
  public readonly candidates: MemoryCandidatesApi;
  public readonly habits: MemoryHabitsApi;
  public readonly retrievals: MemoryRetrievalsApi;
  public readonly contextPacks: MemoryContextPacksApi;
  public readonly feedback: MemoryFeedbackApi;
  public readonly exportJobs: MemoryExportJobsApi;
  public readonly learningSettings: MemoryLearningSettingsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.spaces = new MemorySpacesApi(client);
    this.events = new MemoryEventsApi(client);
    this.sources = new MemorySourcesApi(client);
    this.forgetRequests = new MemoryForgetRequestsApi(client);
    this.extractions = new MemoryExtractionsApi(client);
    this.candidates = new MemoryCandidatesApi(client);
    this.habits = new MemoryHabitsApi(client);
    this.retrievals = new MemoryRetrievalsApi(client);
    this.contextPacks = new MemoryContextPacksApi(client);
    this.feedback = new MemoryFeedbackApi(client);
    this.exportJobs = new MemoryExportJobsApi(client);
    this.learningSettings = new MemoryLearningSettingsApi(client);
  }


async list(params?: MemoryListParams): Promise<MemoryRecordList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'space_id', value: params?.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'memory_type', value: params?.memoryType, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryRecordList>(appendQueryString(appApiPath(`/memory/memories`), query));
  }

async create(body: MemoryRecordRequest, params?: MemoryCreateParams): Promise<MemoryRecord> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryRecord>(appApiPath(`/memory/memories`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(memoryId: string): Promise<MemoryRecord> {
    return this.client.get<MemoryRecord>(appApiPath(`/memory/memories/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}`));
  }

async update(memoryId: string, body: MemoryRecordRequest): Promise<MemoryRecord> {
    return this.client.patch<MemoryRecord>(appApiPath(`/memory/memories/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

async delete(memoryId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/memory/memories/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}`));
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
