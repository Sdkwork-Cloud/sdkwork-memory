export interface MemoryContextPack {
  contextPackId: string;
  retrievalId?: string | null;
  query?: string | null;
  pack: Record<string, unknown>;
  estimatedTokens: number;
  truncated: boolean;
  createdAt: string;
}
