export interface MemoryPcHostAdapter {
  openExternal(url: URL): Promise<void>;
}

export const browserMemoryPcHostAdapter: MemoryPcHostAdapter = {
  async openExternal(url) {
    if (url.protocol !== "https:") throw new Error("External links must use HTTPS");
    window.open(url, "_blank", "noopener,noreferrer");
  },
};
