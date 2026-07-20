import { createMemoryAdminSdkClient } from "@sdkwork/memory-pc-admin-core";
import { createMemoryPcRuntime, loadMemoryPcRuntimeConfig, type MemoryLocale } from "@sdkwork/memory-pc-core";

export async function bootstrapMemoryPcRuntime() {
  const config = await loadMemoryPcRuntimeConfig();
  let locale: MemoryLocale = config.defaultLocale;
  const runtime = createMemoryPcRuntime(config, () => locale);
  const adminClient = createMemoryAdminSdkClient(config.backendApiBaseUrl, runtime.tokenManager);
  return {
    ...runtime,
    adminClient,
    getLocale: () => locale,
    setLocale: (nextLocale: MemoryLocale) => { locale = nextLocale; },
  } as const;
}

export type BootstrappedMemoryPcRuntime = Awaited<ReturnType<typeof bootstrapMemoryPcRuntime>>;
