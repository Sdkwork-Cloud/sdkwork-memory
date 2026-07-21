# Multi-stage build with layer caching optimization.
# Stage 1: Build the release binary with cargo.
FROM rust:1-bookworm AS builder
WORKDIR /workspace/sdkwork-memory

# Copy manifest files first to leverage Docker layer caching for dependency
# compilation.  Only source changes trigger a full rebuild.
COPY sdkwork-memory/Cargo.toml sdkwork-memory/Cargo.lock ./
COPY sdkwork-memory/crates/ ./crates/
COPY sdkwork-memory/plugins/ ./plugins/
COPY sdkwork-memory/database/ ./database/
COPY sdkwork-memory/apis/ ./apis/
COPY sdkwork-memory/sdks/ ./sdks/
COPY sdkwork-memory/etc/ ./etc/
COPY sdkwork-memory/specs/ ./specs/
COPY sdkwork-memory/tools/ ./tools/
COPY sdkwork-memory/docs/ ./docs/
COPY sdkwork-memory/scripts/ ./scripts/
COPY sdkwork-memory/deployments/ ./deployments/
COPY sdkwork-memory/tests/ ./tests/
COPY sdkwork-memory/.env.example sdkwork-memory/.env.postgres.example ./

# Locked local path dependency closure from `cargo metadata --locked`.
COPY sdkwork-database/ /workspace/sdkwork-database/
COPY sdkwork-drive/ /workspace/sdkwork-drive/
COPY sdkwork-iam/ /workspace/sdkwork-iam/
COPY sdkwork-id/ /workspace/sdkwork-id/
COPY sdkwork-utils/ /workspace/sdkwork-utils/
COPY sdkwork-web-framework/ /workspace/sdkwork-web-framework/

# Build the release binary with OpenTelemetry support.
RUN cargo build --release -p sdkwork-api-memory-standalone-gateway --features otel

# Stage 2: Minimal runtime image.
FROM debian:bookworm-slim
ARG SDKWORK_SOURCE_REVISION=unknown
ARG SDKWORK_RELEASE_VERSION=0.0.0-development
LABEL org.opencontainers.image.title="SDKWork Memory" \
      org.opencontainers.image.source="https://github.com/Sdkwork-Cloud/sdkwork-memory" \
      org.opencontainers.image.revision="${SDKWORK_SOURCE_REVISION}" \
      org.opencontainers.image.version="${SDKWORK_RELEASE_VERSION}"
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/sdkwork-memory/target/release/sdkwork-api-memory-standalone-gateway /usr/local/bin/sdkwork-api-memory-standalone-gateway
COPY --from=builder /workspace/sdkwork-memory/database /app/database
ENV SDKWORK_MEMORY_APP_ROOT=/app
ENV SDKWORK_MEMORY_ENVIRONMENT=production
ENV SDKWORK_MEMORY_CONFIG_PROFILE=production
ENV SDKWORK_MEMORY_DEPLOYMENT_PROFILE=cloud
ENV SDKWORK_MEMORY_RUNTIME_TARGET=container
ENV SDKWORK_MEMORY_APPLICATION_PUBLIC_INGRESS_BIND=0.0.0.0:8080
# Default rate limits (overridable at deploy time).
ENV SDKWORK_MEMORY_MAX_BODY_BYTES=1048576
ENV SDKWORK_MEMORY_MAX_CONCURRENCY=256
EXPOSE 8080
USER 10001:10001
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD ["curl", "-sf", "http://localhost:8080/healthz"]
CMD ["sdkwork-api-memory-standalone-gateway"]
