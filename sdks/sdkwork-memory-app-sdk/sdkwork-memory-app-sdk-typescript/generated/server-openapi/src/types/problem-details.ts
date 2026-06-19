export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail?: string | null;
  instance?: string | null;
  code?: string | null;
  requestId?: string | null;
  traceId?: string | null;
  retryable?: boolean | null;
}
