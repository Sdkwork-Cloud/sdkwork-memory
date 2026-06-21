FROM rust:1-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p sdkwork-memory-api-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/sdkwork-memory-api-server /usr/local/bin/sdkwork-memory-api-server
COPY --from=builder /src/database /app/database
ENV SDKWORK_MEMORY_APP_ROOT=/app
ENV SDKWORK_MEMORY_ENVIRONMENT=production
ENV SDKWORK_MEMORY_CONFIG_PROFILE=production
ENV SDKWORK_MEMORY_APPLICATION_PUBLIC_INGRESS_BIND=0.0.0.0:8080
EXPOSE 8080
CMD ["sdkwork-memory-api-server"]
