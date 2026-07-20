import { createContext, useContext, useMemo, type ReactNode } from "react";

import { messages as enUSCommons } from "./en-US/memory/commons/resource-workspace.ts";
import { messages as zhCNCommons } from "./zh-CN/memory/commons/resource-workspace.ts";
import type { MemoryMessageCatalog, MemoryPcModuleDefinition } from "../types.ts";

export type MemoryLocale = "en-US" | "zh-CN";

interface MemoryI18nContextValue {
  locale: MemoryLocale;
  setLocale(locale: MemoryLocale): void;
  translate(key: string): string;
}

const MemoryI18nContext = createContext<MemoryI18nContextValue | null>(null);

export interface MemoryI18nProviderProps {
  children: ReactNode;
  locale: MemoryLocale;
  modules: readonly MemoryPcModuleDefinition[];
  setLocale(locale: MemoryLocale): void;
}

export function MemoryI18nProvider({ children, locale, modules, setLocale }: MemoryI18nProviderProps) {
  const catalog = useMemo(() => buildCatalog(locale, modules), [locale, modules]);
  const value = useMemo<MemoryI18nContextValue>(() => ({
    locale,
    setLocale,
    translate: (key) => catalog[key] ?? key,
  }), [catalog, locale, setLocale]);

  return <MemoryI18nContext.Provider value={value}>{children}</MemoryI18nContext.Provider>;
}

export function useMemoryI18n(): MemoryI18nContextValue {
  const value = useContext(MemoryI18nContext);
  if (!value) throw new Error("MemoryI18nProvider is required");
  return value;
}

function buildCatalog(locale: MemoryLocale, modules: readonly MemoryPcModuleDefinition[]): MemoryMessageCatalog {
  const fallback = enUSCommons;
  const selected = locale === "zh-CN" ? zhCNCommons : enUSCommons;
  return Object.assign({}, fallback, selected, ...modules.map((module) => module.messages["en-US"] ?? {}), ...modules.map((module) => module.messages[locale] ?? {}));
}
