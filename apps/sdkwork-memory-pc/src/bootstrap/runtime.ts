import { createMemoryPcRuntime, loadMemoryPcRuntimeConfig, type MemoryLocale } from "@sdkwork/memory-pc-core";

export async function bootstrapMemoryPcRuntime() {
  const config = await loadMemoryPcRuntimeConfig();
  let locale: MemoryLocale = config.defaultLocale;
  const runtime = createMemoryPcRuntime(config, () => locale);
  return {
    ...runtime,
    getLocale: () => locale,
    setLocale: (nextLocale: MemoryLocale) => { locale = nextLocale; },
  } as const;
}

export type BootstrappedMemoryPcRuntime = Awaited<ReturnType<typeof bootstrapMemoryPcRuntime>>;
