import type { MemoryPageInfo, MemoryPageResult } from "../types.ts";

export function normalizeMemoryPage(value: unknown): MemoryPageResult {
  if (!isRecord(value)) return emptyMemoryPage();
  const data = isRecord(value.data) ? value.data : value;
  const items = Array.isArray(data.items) ? data.items.filter(isRecord) : [];
  const rawPageInfo = isRecord(data.pageInfo) ? data.pageInfo : {};
  return {
    items,
    pageInfo: normalizePageInfo(rawPageInfo),
  };
}

export function normalizeMemoryItem(value: unknown): MemoryPageResult {
  if (!isRecord(value)) return emptyMemoryPage();
  const data = isRecord(value.data) ? value.data : value;
  const item = isRecord(data.item) ? data.item : data;
  return {
    items: isRecord(item) ? [item] : [],
    pageInfo: { mode: "offset", page: 1, pageSize: 1, total: isRecord(item) ? 1 : 0 },
  };
}

export function emptyMemoryPage(): MemoryPageResult {
  return { items: [], pageInfo: { mode: "cursor", hasNext: false } };
}

function normalizePageInfo(value: Record<string, unknown>): MemoryPageInfo {
  const mode = value.mode === "offset" ? "offset" : "cursor";
  return {
    mode,
    ...(typeof value.cursor === "string" ? { cursor: value.cursor } : {}),
    ...(typeof value.nextCursor === "string" ? { nextCursor: value.nextCursor } : {}),
    ...(typeof value.hasNext === "boolean" ? { hasNext: value.hasNext } : {}),
    ...(typeof value.page === "number" ? { page: value.page } : {}),
    ...(typeof value.pageSize === "number" ? { pageSize: value.pageSize } : {}),
    ...(typeof value.total === "number" ? { total: value.total } : {}),
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
