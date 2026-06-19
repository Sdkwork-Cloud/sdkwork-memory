export interface MemoryReviewRequest {
  reason?: string | null;
  reviewerNote?: string | null;
  metadata?: Record<string, unknown> | null;
}
