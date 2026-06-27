# ADR-001: Background Worker Graceful Shutdown

- Status: Accepted
- Date: 2026-06-27
- Deciders: Memory Platform Team

## Context

The `spawn_background_workers` function in `job_worker.rs` used `std::mem::forget(shutdown_tx)` to discard the `watch::Sender<bool>`. This prevented the application from triggering a clean shutdown of background workers (outbox publisher, learning job worker, eval run worker, provider health probe). Workers could only terminate when the process exited, risking data loss for in-flight outbox deliveries and job processing.

## Decision

Return the `watch::Sender<bool>` from `spawn_background_workers` and propagate it through `build_router` as `MemoryApplication::worker_shutdown_tx`. In `main.rs`, trigger `send(true)` during the graceful shutdown signal handler, then wait a bounded grace period (3 seconds) for workers to drain.

## Consequences

- **Positive**: Workers receive an explicit shutdown signal and can log confirmation before exiting. In-flight work is drained, reducing data loss risk.
- **Positive**: The shutdown handle is part of the application contract, making it testable.
- **Negative**: The 3-second grace period may be insufficient for long-running jobs. Future work should make this configurable and per-worker.
- **Mitigation**: Workers already use `tokio::select!` with the shutdown channel, so they exit promptly even mid-iteration.
