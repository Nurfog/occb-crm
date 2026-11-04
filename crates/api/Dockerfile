# API OCCB — imagen multi-etapa (compilación + ejecución mínima).

# ---------- Etapa 1: compilación ----------
FROM rust:1.98-trixie AS compilacion

WORKDIR /codigo
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml ./crates/core/Cargo.toml
COPY crates/api/Cargo.toml ./crates/api/Cargo.toml
COPY crates/app/Cargo.toml ./crates/app/Cargo.toml
COPY crates ./crates

RUN cargo build --release -p occb-api

# ---------- Etapa 2: ejecución ----------
FROM debian:trixie-slim AS ejecucion

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=compilacion /codigo/target/release/occb-api /usr/local/bin/occb-api

EXPOSE 3001
CMD ["occb-api"]
