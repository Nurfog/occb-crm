# Frontend OCCB CRM (Dioxus, solo frontend).
# El API lo provee el backend del proyecto occb.
#
# Build: docker build -t occb-crm-frontend \
#   --build-arg API_BASE_URL=https://api.tu-dominio.com \
#   --build-arg PRIVACY_CONTACT_EMAIL=privacidad@tu-dominio.com .

FROM rust:1.98-bookworm AS build
ARG API_BASE_URL=http://127.0.0.1:8000
ENV API_BASE_URL=${API_BASE_URL}
ARG PRIVACY_CONTACT_EMAIL=privacidad@example.com
ENV PRIVACY_CONTACT_EMAIL=${PRIVACY_CONTACT_EMAIL}
RUN rustup target add wasm32-unknown-unknown \
    && cargo install dioxus-cli --version 0.7.10 --locked
WORKDIR /app
COPY . .
RUN dx build --release

FROM nginx:1.27-alpine
# `dx build --release` deja el sitio en target/dx/<pkg>/release/web/public.
COPY --from=build /app/target/dx/occb-crm-frontend/release/web/public /usr/share/nginx/html
# Fallback SPA: el router del cliente resuelve /privacidad, /derechos, etc.
COPY nginx.conf /etc/nginx/conf.d/default.conf
# En producción, proxear /api/* al backend occb o servir con API_BASE_URL absoluta.
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
