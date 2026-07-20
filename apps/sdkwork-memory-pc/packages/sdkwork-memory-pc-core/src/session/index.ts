import { createSdkworkIamRuntimeAuthController, type SdkworkIamRuntimeAuthRuntimeLike } from "@sdkwork/auth-pc-react";
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
      deploymentMode: config.deploymentProfile === "cloud" ? "saas" : "local",
      environment: config.environment === "test" ? "test" : config.environment === "development" ? "dev" : "prod",
      platform: "pc",
    },
    baseUrls: { appbaseAppApiBaseUrl: config.appbaseAppApiBaseUrl },
    localeProvider,
    sdkClients: [appClient],
    sessionAuth: true,
    tokenManager,
  });
  // IAM's controller port intentionally accepts a narrower structural runtime than IamRuntime.
  const getAuthRuntime = () => auth.getRuntime() as unknown as SdkworkIamRuntimeAuthRuntimeLike;
  const authController = createSdkworkIamRuntimeAuthController({ getRuntime: getAuthRuntime });
  return { appClient, auth, authController, config, tokenManager } as const;
}

export type MemoryPcRuntime = ReturnType<typeof createMemoryPcRuntime>;
