import { createSdkworkIamRuntimeAuthController } from "@sdkwork/auth-pc-react";
import { createSdkworkAppbasePcAuthRuntime } from "@sdkwork/auth-runtime-pc-react";
import { createTokenManager } from "@sdkwork/sdk-common";

import type { MemoryPcRuntimeConfig } from "../config/runtime-config.ts";
import { createMemoryAppSdkClient } from "../sdk/index.ts";

export function createMemoryPcRuntime(config: MemoryPcRuntimeConfig, localeProvider: () => string | undefined) {
  const tokenManager = createTokenManager();
  const appClient = createMemoryAppSdkClient({ baseUrl: config.appApiBaseUrl, tokenManager });
  const auth = createSdkworkAppbasePcAuthRuntime({
    app: {
      appId: "sdkwork-memory-pc",
      deploymentMode: config.deploymentProfile,
      environment: config.environment,
      platform: "pc",
    },
    baseUrls: { appbaseAppApiBaseUrl: config.appbaseAppApiBaseUrl },
    localeProvider,
    sdkClients: [appClient],
    sessionAuth: true,
    tokenManager,
  });
  const authController = createSdkworkIamRuntimeAuthController({ getRuntime: auth.getRuntime });
  return { appClient, auth, authController, config, tokenManager } as const;
}

export type MemoryPcRuntime = ReturnType<typeof createMemoryPcRuntime>;
