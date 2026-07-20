import { AlertTriangle, ChevronLeft, ChevronRight, Database, RefreshCw, Search, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { useMemoryI18n } from "../i18n/runtime.tsx";
import type { MemoryListQuery, MemoryPageResult, MemoryPcModuleDefinition, MemoryPcResourceKey, MemoryResourceRegistry } from "../types.ts";

const DEFAULT_PAGE: MemoryPageResult = { items: [], pageInfo: { mode: "cursor", hasNext: false } };

export interface MemoryModulePageProps {
  module: MemoryPcModuleDefinition;
  registry: MemoryResourceRegistry;
}

export function MemoryModulePage({ module, registry }: MemoryModulePageProps) {
  const { translate } = useMemoryI18n();
  const [resource, setResource] = useState<MemoryPcResourceKey>(module.resources[0] ?? "spaces");
  const [q, setQ] = useState("");
  const [spaceId, setSpaceId] = useState("");
  const [cursor, setCursor] = useState<string>();
  const [cursorHistory, setCursorHistory] = useState<string[]>([]);
  const [pageSize, setPageSize] = useState(20);
  const [refreshVersion, setRefreshVersion] = useState(0);
  const [page, setPage] = useState(DEFAULT_PAGE);
  const [selectedItem, setSelectedItem] = useState<Record<string, unknown> | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>();
  const dataSource = registry[resource];

  useEffect(() => {
    setCursor(undefined);
    setCursorHistory([]);
    setSelectedItem(null);
  }, [resource, q, spaceId, pageSize]);

  useEffect(() => {
    if (!dataSource) {
      setPage(DEFAULT_PAGE);
      setLoading(false);
      setError(undefined);
      return;
    }
    const controller = new AbortController();
    const query: MemoryListQuery = {
      pageSize,
      ...(q.trim() ? { q: q.trim() } : {}),
      ...(spaceId.trim() ? { spaceId: spaceId.trim() } : {}),
      ...(cursor ? { cursor } : {}),
    };
    setLoading(true);
    setError(undefined);
    void dataSource.load(query, controller.signal).then((result) => {
      if (!controller.signal.aborted) setPage(result);
    }).catch((reason: unknown) => {
      if (!controller.signal.aborted) setError(readSafeError(reason));
    }).finally(() => {
      if (!controller.signal.aborted) setLoading(false);
    });
    return () => controller.abort();
  }, [cursor, dataSource, pageSize, q, refreshVersion, spaceId]);

  const columns = useMemo(() => resolveColumns(page.items), [page.items]);
  const nextCursor = page.pageInfo.nextCursor;

  function openNextPage(): void {
    if (!nextCursor) return;
    setCursorHistory((history) => [...history, cursor ?? ""]);
    setCursor(nextCursor);
  }

  function openPreviousPage(): void {
    setCursorHistory((history) => {
      const previous = history.at(-1);
      setCursor(previous || undefined);
      return history.slice(0, -1);
    });
  }

  return (
    <section className="module-page" aria-labelledby="module-title">
      <header className="module-header">
        <div>
          <p className="module-kicker">{module.surface === "backend-admin" ? translate("memory.commons.operatorSurface") : translate("memory.commons.userSurface")}</p>
          <h1 id="module-title">{translate(module.titleKey)}</h1>
        </div>
        <button className="icon-button" type="button" title={translate("memory.commons.refresh")} aria-label={translate("memory.commons.refresh")} onClick={() => setRefreshVersion((version) => version + 1)}>
          <RefreshCw size={17} aria-hidden="true" />
        </button>
      </header>

      <div className="resource-tabs" role="tablist" aria-label={translate(module.titleKey)}>
        {module.resources.map((candidate) => (
          <button key={candidate} type="button" role="tab" aria-selected={candidate === resource} className={candidate === resource ? "resource-tab active" : "resource-tab"} onClick={() => setResource(candidate)}>
            {formatResourceName(candidate)}
          </button>
        ))}
      </div>

      <div className="resource-toolbar">
        <label className="field-control search-field">
          <span>{translate("memory.commons.search")}</span>
          <div><Search size={16} aria-hidden="true" /><input value={q} onChange={(event) => setQ(event.target.value)} placeholder={translate("memory.commons.searchPlaceholder")} /></div>
        </label>
        <label className="field-control scope-field">
          <span>{translate("memory.commons.spaceId")}</span>
          <input value={spaceId} onChange={(event) => setSpaceId(event.target.value)} placeholder={translate("memory.commons.spaceIdPlaceholder")} />
        </label>
        <label className="field-control page-size-field">
          <span>{translate("memory.commons.pageSize")}</span>
          <select value={pageSize} onChange={(event) => setPageSize(Number(event.target.value))}>
            <option value={20}>20</option>
            <option value={50}>50</option>
            <option value={100}>100</option>
          </select>
        </label>
      </div>

      <div className="resource-stage" aria-live="polite" aria-busy={loading}>
        {!dataSource ? <StatusState icon={<Database size={24} />} message={translate("memory.commons.unavailable")} /> : null}
        {dataSource && loading ? <div className="loading-line"><span className="spinner" />{translate("memory.commons.loading")}</div> : null}
        {dataSource && error ? <StatusState icon={<AlertTriangle size={24} />} message={`${translate("memory.commons.error")} ${error}`} tone="danger" /> : null}
        {dataSource && !loading && !error && page.items.length === 0 ? <StatusState icon={<Database size={24} />} message={translate("memory.commons.empty")} /> : null}
        {dataSource && !loading && !error && page.items.length > 0 ? (
          <div className="table-scroll">
            <table>
              <thead><tr>{columns.map((column) => <th key={column}>{formatResourceName(column)}</th>)}</tr></thead>
              <tbody>{page.items.map((item, rowIndex) => (
                <tr key={readRowKey(item, rowIndex)} tabIndex={0} onClick={() => setSelectedItem(item)} onKeyDown={(event) => { if (event.key === "Enter") setSelectedItem(item); }}>
                  {columns.map((column) => <td key={column}>{formatValue(item[column])}</td>)}
                </tr>
              ))}</tbody>
            </table>
          </div>
        ) : null}
      </div>

      <footer className="pagination-bar">
        <span>{typeof page.pageInfo.total === "number" ? page.pageInfo.total.toLocaleString() : `${page.items.length}`}</span>
        <div>
          <button className="icon-button" type="button" disabled={cursorHistory.length === 0} title={translate("memory.commons.previous")} aria-label={translate("memory.commons.previous")} onClick={openPreviousPage}><ChevronLeft size={17} /></button>
          <button className="icon-button" type="button" disabled={!nextCursor} title={translate("memory.commons.next")} aria-label={translate("memory.commons.next")} onClick={openNextPage}><ChevronRight size={17} /></button>
        </div>
      </footer>

      {selectedItem ? (
        <aside className="detail-drawer" aria-label={translate("memory.commons.details")}>
          <header><h2>{translate("memory.commons.details")}</h2><button className="icon-button" type="button" title={translate("memory.commons.close")} aria-label={translate("memory.commons.close")} onClick={() => setSelectedItem(null)}><X size={17} /></button></header>
          <dl>{Object.entries(selectedItem).map(([key, value]) => <div key={key}><dt>{formatResourceName(key)}</dt><dd>{formatLongValue(value)}</dd></div>)}</dl>
        </aside>
      ) : null}
    </section>
  );
}

function StatusState({ icon, message, tone = "neutral" }: { icon: React.ReactNode; message: string; tone?: "danger" | "neutral" }) {
  return <div className={`status-state ${tone}`}>{icon}<p>{message}</p></div>;
}

function resolveColumns(items: readonly Record<string, unknown>[]): string[] {
  const preferred = ["id", "uuid", "displayName", "name", "title", "content", "status", "state", "type", "updatedAt", "createdAt"];
  const keys = new Set(items.flatMap((item) => Object.keys(item)));
  return [...preferred.filter((key) => keys.has(key)), ...[...keys].filter((key) => !preferred.includes(key))].slice(0, 7);
}

function readRowKey(item: Record<string, unknown>, index: number): string {
  const value = item.id ?? item.uuid;
  return typeof value === "string" || typeof value === "number" ? String(value) : `row-${index}`;
}

function formatResourceName(value: string): string {
  return value.replace(/([a-z0-9])([A-Z])/g, "$1 $2").replace(/[-_]/g, " ").replace(/^./, (letter) => letter.toUpperCase());
}

function formatValue(value: unknown): string {
  const text = formatLongValue(value);
  return text.length > 96 ? `${text.slice(0, 93)}...` : text;
}

function formatLongValue(value: unknown): string {
  if (value === null || value === undefined) return "-";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return JSON.stringify(value, null, 2);
}

function readSafeError(value: unknown): string {
  if (typeof value !== "object" || value === null) return "";
  const error = value as { code?: unknown; traceId?: unknown };
  const code = typeof error.code === "number" ? `Code ${error.code}.` : "";
  const trace = typeof error.traceId === "string" ? ` Trace ${error.traceId}.` : "";
  return `${code}${trace}`.trim();
}
