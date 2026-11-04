# Frontal OCCB (Dioxus web) — imagen multi-etapa.
# Etapa 1: compila el WASM con Dioxus CLI. Etapa 2: sirve los estáticos con nginx.

# ---------- Etapa 1: compilación ----------
FROM rust:1.98-trixie AS compilacion

# Herramientas mínimas para compilar Rust a WASM.
RUN apt-get update \
  && apt-get install -y --no-install-recommends curl ca-certificates pkg-config libssl-dev \
  && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown

# Dioxus CLI (última estable 0.7, misma que en desarrollo).
RUN curl -fsSL https://dioxus.dev/install.sh | bash
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /codigo

# Primero solo manifiestos para aprovechar la caché de dependencias.
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml ./crates/core/Cargo.toml
COPY crates/api/Cargo.toml ./crates/api/Cargo.toml
COPY crates/app/Cargo.toml ./crates/app/Cargo.toml
COPY crates/app/Dioxus.toml ./crates/app/Dioxus.toml

# Copia el código del frontal y sus assets.
COPY crates/core ./crates/core
COPY crates/api ./crates/api
COPY crates/app ./crates/app

WORKDIR /codigo/crates/app
RUN dx build --web @client --release

# ---------- Etapa 2: servido ----------
FROM nginx:1.27-alpine AS servido

COPY crates/app/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=compilacion /codigo/target/dx/occb-app/release/web/public /usr/share/nginx/html

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
