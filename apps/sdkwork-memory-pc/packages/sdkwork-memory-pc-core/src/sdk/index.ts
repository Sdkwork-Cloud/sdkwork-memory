import { createClient, type SdkworkAppClient } from "@sdkwork/memory-app-sdk";
import type { AuthTokenManager } from "@sdkwork/sdk-common";

export type MemoryAppSdkClient = SdkworkAppClient;

export interface CreateMemoryAppSdkClientOptions {
  baseUrl: string;
  tokenManager: AuthTokenManager;
}

export function createMemoryAppSdkClient({ baseUrl, tokenManager }: CreateMemoryAppSdkClientOptions): MemoryAppSdkClient {
  return createClient({ baseUrl, authMode: "dual-token", platform: "pc", tokenManager });
}
