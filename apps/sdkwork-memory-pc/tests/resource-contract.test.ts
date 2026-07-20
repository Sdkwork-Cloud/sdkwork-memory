import { describe, expect, it, vi } from "vitest";

import { createMemoryAdminResourceRegistry, type MemoryAdminSdkClient } from "@sdkwork/memory-pc-admin-core";
import { hasPermissionHint, normalizeMemoryPage } from "@sdkwork/memory-pc-commons";
import { createMemoryConsoleResourceRegistry, type MemoryConsoleSdkClient } from "@sdkwork/memory-pc-console-core";

describe("Memory resource contracts", () => {
  it("normalizes SDKWork list data without client-side pagination", () => {
    const result = normalizeMemoryPage({ items: [{ id: "1" }, { id: "2" }], pageInfo: { mode: "cursor", nextCursor: "next" } });
    expect(result.items).toHaveLength(2);
    expect(result.pageInfo.nextCursor).toBe("next");
  });

  it("passes console pagination to the generated app SDK", async () => {
    const list = vi.fn().mockResolvedValue({ items: [], pageInfo: { mode: "cursor" } });
    const client = { memory: { spaces: { list } } } as unknown as MemoryConsoleSdkClient;
    const source = createMemoryConsoleResourceRegistry(client).spaces;
    await source?.load({ q: "preference", cursor: "opaque", pageSize: 50 });
    expect(list).toHaveBeenCalledWith({ q: "preference", cursor: "opaque", pageSize: 50 });
  });

  it("passes admin pagination to the generated backend SDK", async () => {
    const list = vi.fn().mockResolvedValue({ items: [], pageInfo: { mode: "cursor" } });
    const client = { memory: { auditLogs: { list } } } as unknown as MemoryAdminSdkClient;
    const source = createMemoryAdminResourceRegistry(client).auditLogs;
    await source?.load({ q: "policy", cursor: "opaque", pageSize: 20 });
    expect(list).toHaveBeenCalledWith({ q: "policy", cursor: "opaque", pageSize: 20 });
  });
});

describe("Memory route permission hints", () => {
  it("supports exact and prefix wildcard permission hints", () => {
    expect(hasPermissionHint(["memory.records.read"], "memory.records.read")).toBe(true);
    expect(hasPermissionHint(["memory.backend.*"], "memory.backend.auditLogs.read")).toBe(true);
    expect(hasPermissionHint(["memory.records.read"], "memory.backend.records.read")).toBe(false);
  });
});
