export interface MemoryForgetJob {
  forgetRequestId: string;
  state: 'queued' | 'running' | 'succeeded' | 'failed' | 'cancelled';
  result?: Record<string, unknown> | null;
  createdAt: string;
  updatedAt: string;
}
