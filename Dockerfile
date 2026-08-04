FROM node:22-bookworm-slim AS web-build
WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/web/package.json apps/web/package.json
COPY packages packages
RUN corepack enable && corepack prepare pnpm@11.1.2 --activate
RUN pnpm install --frozen-lockfile
COPY apps/web apps/web
RUN pnpm --filter @schema-pathfinder/web build

FROM rust:1.88-bookworm AS api-build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY apps/cli-rust apps/cli-rust
COPY apps/desktop apps/desktop
RUN cargo build --release -p schema-pathfinder --features api --bin schema-pathfinder-api

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update   && apt-get install -y --no-install-recommends ca-certificates   && rm -rf /var/lib/apt/lists/*
COPY --from=api-build /app/target/release/schema-pathfinder-api /usr/local/bin/schema-pathfinder-api
COPY --from=web-build /app/apps/web/dist /app/web
COPY fixtures fixtures
ENV SCHEMA_PATHFINDER_BIND=0.0.0.0:8787
ENV SCHEMA_PATHFINDER_WEB_DIR=/app/web
EXPOSE 8787
CMD ["schema-pathfinder-api"]
