//! Sesión de autenticación en memoria (contexto global).
//!
//! Los tokens viven solo en un `Signal` en memoria: nunca se escriben en
//! `localStorage` (ante XSS, el almacenamiento persistente expone la sesión).
//! Al recargar la página la sesión se pierde y hay que volver a entrar — es
//! el compromiso que pide el backend `occb` hasta que use cookies HttpOnly.

use dioxus::prelude::*;

use crate::api::{TokenPair, UserPublic};

/// Sesión activa: usuario + tokens del último `TokenPair` recibido.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub user: UserPublic,
    pub access_token: String,
    pub refresh_token: String,
}

impl Session {
    pub fn from_pair(pair: TokenPair) -> Self {
        Self {
            user: pair.user,
            access_token: pair.access_token,
            refresh_token: pair.refresh_token,
        }
    }
}

/// Estado global de autenticación.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AuthState {
    pub session: Option<Session>,
}

/// Registra el contexto. Llamar una sola vez, en el componente raíz.
pub fn provide_auth() {
    use_context_provider(|| Signal::new(AuthState::default()));
}

/// Acceso al estado de sesión desde cualquier página o componente.
pub fn use_auth() -> Signal<AuthState> {
    use_context::<Signal<AuthState>>()
}
