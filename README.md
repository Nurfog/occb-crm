# OCCB CRM — frontend (Dioxus)

Frontend del CRM, escrito con [Dioxus](https://dioxuslabs.com) (Rust → Web).
**Solo frontend**: el API vive en el proyecto `occb` (Axum + Postgres, en la
carpeta de workspaces de Orca). La referencia funcional/histórica del CRM
completo (Next.js + Rust) está en `/home/juan/dev/opencrmccb`.

## Requisitos

- Rust estable + target `wasm32-unknown-unknown`
- CLI `dx` 0.7 (`dx --version`)
- Backend `occb` corriendo (por defecto en `http://127.0.0.1:8000`)

```bash
rustup target add wasm32-unknown-unknown
```

## Desarrollo

```bash
dx serve
```

Abre la URL que indique `dx` (normalmente `http://127.0.0.1:8080`).

### Apuntar a otro backend

La URL del API se fija en tiempo de compilación con `API_BASE_URL`:

```bash
API_BASE_URL=http://127.0.0.1:8000 dx serve
```

Por defecto es `http://127.0.0.1:8000`. Como el frontend llama al backend por
URL absoluta, el backend debe permitir el origen del dev-server en
`CORS_ORIGINS` (proyecto `occb`, p. ej.
`CORS_ORIGINS=http://127.0.0.1:8080,http://localhost:8080`).

## Build de producción

```bash
API_BASE_URL=https://api.tu-dominio.com dx build --release
```

Sirve `dist/` con cualquier estático (Nginx/Caddy). Ver `Dockerfile`.

## Rutas

| Ruta | Descripción |
|------|-------------|
| `/` | Portada + estado del backend |
| `/login` | Entrada (`POST /api/v1/auth/login`) |
| `/register` | Registro (`POST /api/v1/auth/register`) |
| `/dashboard` | Perfil (`GET /api/v1/auth/me`), renovar tokens, salir |
| `/users` | Listado con búsqueda/orden/paginación (`GET /api/v1/users`) |
| `/users/:id` | Detalle (`GET /api/v1/users/:id`) |
| `/status` | `health`, `ready` y `version` del backend |
| `/privacidad` | Política de Privacidad (Ley 21.719) |
| `/derechos` | Canal de derechos ARCOP+B + portabilidad |

## Endpoints cubiertos

Auth con rotación de refresh tokens (`register`, `login`, `refresh`,
`logout`, `me` con perfil extendido, `profile` PUT para rectificación
directa), usuarios (`get`; el listado `GET /users` es solo admin y la UI lo
indica), observabilidad (`health`, `ready`, `version`) y privacidad Ley 21.719
(`privacy/requests` POST público + verificación, `privacy/consents` POST/GET
latest con sesión). Los módulos CRM de la referencia `opencrmccb`
(contactos, empresas, deals, actividades, documentos, reportes…) se añadirán
en `src/api.rs` + `src/pages/` cuando el backend `occb` los exponga.

## Seguridad

- Tokens **solo en memoria** (Signal global): nunca en `localStorage`.
- Salida: `POST /api/v1/auth/logout` con el access token en `Authorization`
  y el refresh en el cuerpo; la sesión local se cierra aunque falle la red.
- Los errores muestran `{ "error": { "code", "message" } }` del backend, sin
  filtrar detalles internos (el backend ya los oculta).

## Privacidad — Ley 21.719 (Chile)

Ver el mapa completo en [`docs/ley-21719.md`](./docs/ley-21719.md)
(no es asesoría legal). El frontend aporta:

- Banner de consentimiento **opt-in y granular** (marketing/analítica
  desactivados por defecto, revocable en «Preferencias de privacidad»).
- Aceptación de la Política de Privacidad al registrarse + evidencia local
  (fecha y versión) del consentimiento.
- Canal de derechos **ARCOP+B** (`/derechos`) con plazos legales visibles
  (30 días corridos; bloqueo en 2 días hábiles) y portabilidad en JSON.
- `PRIVACY_CONTACT_EMAIL` (build-time) define el canal formal mientras el
  backend implementa `POST /api/v1/privacy/requests` (hoy: acuse provisorio).

## Estructura

```
src/
├── main.rs          # raíz + Router
├── route.rs         # mapa de rutas
├── config.rs        # API_BASE_URL
├── api.rs           # cliente del API occb
├── auth.rs          # sesión en memoria (contexto)
├── components.rs    # NavBar + Layout
└── pages/           # home, auth, dashboard, users, status, 404
assets/main.css      # estilos planos, sin Tailwind
```
