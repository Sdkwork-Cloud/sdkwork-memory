import { describe, expect, it } from "vitest";

import { parseMemoryPcRuntimeConfig } from "@sdkwork/memory-pc-core";

const validConfig = {
  environment: "production",
  deploymentProfile: "cloud",
  appApiBaseUrl: "https://memory-app.sdkwork.com",
  backendApiBaseUrl: "https://memory-admin.sdkwork.com",
  appbaseAppApiBaseUrl: "https://api.sdkwork.com",
  defaultLocale: "zh-CN",
  fallbackLocale: "en-US",
  supportedLocales: ["zh-CN", "en-US"],
};

describe("Memory PC runtime config", () => {
  it("accepts independent public SDK surface URLs", () => {
    const config = parseMemoryPcRuntimeConfig(validConfig);
    expect(config.appApiBaseUrl).toBe("https://memory-app.sdkwork.com");
    expect(config.backendApiBaseUrl).toBe("https://memory-admin.sdkwork.com");
  });

  it("rejects production loopback endpoints", () => {
    expect(() => parseMemoryPcRuntimeConfig({ ...validConfig, appApiBaseUrl: "http://127.0.0.1:8080" })).toThrow(/loopback/);
  });

  it("rejects unsupported locale fallbacks", () => {
    expect(() => parseMemoryPcRuntimeConfig({ ...validConfig, supportedLocales: ["zh-CN"] })).toThrow(/supported/);
  });
});
