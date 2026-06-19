import { customApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { MemoryCandidate, MemoryCandidateList, MemoryCapabilities, MemoryContextPack, MemoryContextPackRequest, MemoryEvent, MemoryEventRequest, MemoryExtractionRequest, MemoryFeedback, MemoryFeedbackRequest, MemoryLearningJob, MemoryProviderHealth, MemoryRecord, MemoryRecordList, MemoryRecordRequest, MemoryRetrievalRequest, MemoryRetrievalResult } from '../types';


export class MemoryProviderHealthApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<MemoryProviderHealth> {
    return this.client.get<MemoryProviderHealth>(customApiPath(`/memory/provider_health`));
  }
}

export interface MemoryCandidatesListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
  decisionState?: string;
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
    return this.client.get<MemoryCandidateList>(appendQueryString(customApiPath(`/memory/candidates`), query));
  }

async retrieve(candidateId: string): Promise<MemoryCandidate> {
    return this.client.get<MemoryCandidate>(customApiPath(`/memory/candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}`));
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
    return this.client.post<MemoryLearningJob>(customApiPath(`/memory/extractions`), body, undefined, requestHeaders, 'application/json');
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
    return this.client.post<MemoryFeedback>(customApiPath(`/memory/feedback`), body, undefined, requestHeaders, 'application/json');
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
    return this.client.post<MemoryContextPack>(customApiPath(`/memory/context_packs`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(contextPackId: string): Promise<MemoryContextPack> {
    return this.client.get<MemoryContextPack>(customApiPath(`/memory/context_packs/${serializePathParameter(contextPackId, { name: 'contextPackId', style: 'simple', explode: false })}`));
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
    return this.client.post<MemoryRetrievalResult>(customApiPath(`/memory/retrievals`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(retrievalId: string): Promise<MemoryRetrievalResult> {
    return this.client.get<MemoryRetrievalResult>(customApiPath(`/memory/retrievals/${serializePathParameter(retrievalId, { name: 'retrievalId', style: 'simple', explode: false })}`));
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
    return this.client.post<MemoryEvent>(customApiPath(`/memory/events`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(eventId: string): Promise<MemoryEvent> {
    return this.client.get<MemoryEvent>(customApiPath(`/memory/events/${serializePathParameter(eventId, { name: 'eventId', style: 'simple', explode: false })}`));
  }
}

export class MemoryCapabilitiesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


async retrieve(): Promise<MemoryCapabilities> {
    return this.client.get<MemoryCapabilities>(customApiPath(`/memory/capabilities`));
  }
}

export interface MemoryListParams {
  q?: string;
  cursor?: string;
  pageSize?: number;
  spaceId?: string;
  memoryType?: string;
  externalSubjectRef?: string;
}

export interface MemoryCreateParams {
  idempotencyKey?: string;
}

export class MemoryApi {
  private client: HttpClient;
  public readonly capabilities: MemoryCapabilitiesApi;
  public readonly events: MemoryEventsApi;
  public readonly retrievals: MemoryRetrievalsApi;
  public readonly contextPacks: MemoryContextPacksApi;
  public readonly feedback: MemoryFeedbackApi;
  public readonly extractions: MemoryExtractionsApi;
  public readonly candidates: MemoryCandidatesApi;
  public readonly providerHealth: MemoryProviderHealthApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.capabilities = new MemoryCapabilitiesApi(client);
    this.events = new MemoryEventsApi(client);
    this.retrievals = new MemoryRetrievalsApi(client);
    this.contextPacks = new MemoryContextPacksApi(client);
    this.feedback = new MemoryFeedbackApi(client);
    this.extractions = new MemoryExtractionsApi(client);
    this.candidates = new MemoryCandidatesApi(client);
    this.providerHealth = new MemoryProviderHealthApi(client);
  }


async list(params?: MemoryListParams): Promise<MemoryRecordList> {
    const query = buildQueryString([
      { name: 'q', value: params?.q, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
      { name: 'space_id', value: params?.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'memory_type', value: params?.memoryType, style: 'form', explode: true, allowReserved: false },
      { name: 'external_subject_ref', value: params?.externalSubjectRef, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MemoryRecordList>(appendQueryString(customApiPath(`/memory/memories`), query));
  }

async create(body: MemoryRecordRequest, params?: MemoryCreateParams): Promise<MemoryRecord> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params?.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<MemoryRecord>(customApiPath(`/memory/memories`), body, undefined, requestHeaders, 'application/json');
  }

async retrieve(memoryId: string): Promise<MemoryRecord> {
    return this.client.get<MemoryRecord>(customApiPath(`/memory/memories/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}`));
  }

async update(memoryId: string, body: MemoryRecordRequest): Promise<MemoryRecord> {
    return this.client.patch<MemoryRecord>(customApiPath(`/memory/memories/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

async delete(memoryId: string): Promise<void> {
    return this.client.delete<void>(customApiPath(`/memory/memories/${serializePathParameter(memoryId, { name: 'memoryId', style: 'simple', explode: false })}`));
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
