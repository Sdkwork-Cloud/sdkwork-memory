export interface MemoryRecordSource {
  sourceId: string;
  memoryId: string;
  eventId: string;
  sourceRole: string;
  confidenceDelta?: number | null;
  createdAt: string;
}
