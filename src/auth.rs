use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::{
    extract::State,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::{cookie::Key, PrivateCookieJar};
use cookie::Cookie;
use rand::Rng;
use serde::Deserialize;

use crate::{error::AppError, AppState};

#[derive(Clone)]
pub struct AuthUser {
    pub id: i64,
    pub username: String,
}

#[derive(sqlx::FromRow)]
struct SessionUser {
    id: i64,
    username: String,
}

pub fn generate_csrf_token() -> String {
    let bytes: [u8; 16] = rand::thread_rng().gen();
    hex::encode(bytes)
}

fn cookie_name() -> &'static str {
    "session"
}

pub fn make_session_cookie(user_id: i64, csrf: &str, secure: bool) -> Cookie<'static> {
    let value = format!("{user_id}:{csrf}");
    Cookie::build((cookie_name(), value))
        .http_only(true)
        .same_site(cookie::SameSite::Lax)
        .secure(secure)
        .max_age(cookie::time::Duration::days(7))
        .path("/")
        .build()
}

pub fn parse_session_cookie(value: &str) -> Option<(i64, String)> {
    let (id_str, csrf) = value.split_once(':')?;
    let id: i64 = id_str.parse().ok()?;
    Some((id, csrf.to_string()))
}

impl axum::extract::FromRequestParts<AppState> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar: PrivateCookieJar<Key> =
            PrivateCookieJar::from_request_parts(parts, state)
                .await
                .map_err(|e| AppError::Internal(Box::new(e) as _).into_response())?;

        let cookie = jar
            .get(cookie_name())
            .ok_or_else(|| Redirect::to("/login").into_response())?;

        let (user_id, _csrf) = parse_session_cookie(cookie.value())
            .filter(|(id, _)| *id > 0)
            .ok_or_else(|| Redirect::to("/login").into_response())?;

        let user = sqlx::query_as::<_, SessionUser>("SELECT id, username FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Internal(Box::new(e) as _).into_response())?
            .ok_or_else(|| Redirect::to("/login").into_response())?;

        Ok(AuthUser {
            id: user.id,
            username: user.username,
        })
    }
}

pub fn get_session_csrf(jar: &PrivateCookieJar) -> Option<String> {
    let cookie = jar.get(cookie_name())?;
    let (_id, csrf) = parse_session_cookie(cookie.value())?;
    Some(csrf)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

const DUMMY_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHRzb21lc2FsdA$RdescudvJCsgt3ub+b+dWRWJTmaaJObG";

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[derive(sqlx::FromRow)]
struct LoginRow {
    id: i64,
    password_hash: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub _csrf: Option<String>,
}

pub async fn handle_post_login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Form(form): Form<LoginForm>,
) -> Response {
    let expected_csrf = get_session_csrf(&jar);
    let submitted_csrf = form._csrf.as_deref().unwrap_or("");
    let csrf_ok = expected_csrf
        .as_deref()
        .map(|e| constant_time_eq(e, submitted_csrf))
        .unwrap_or(false);

    if !csrf_ok {
        return (
            StatusCode::FORBIDDEN,
            crate::views::login_page("", submitted_csrf, Some("Invalid CSRF token.")),
        )
            .into_response();
    }

    let row =
        sqlx::query_as::<_, LoginRow>("SELECT id, password_hash FROM users WHERE username = ?")
            .bind(&form.username)
            .fetch_optional(&state.pool)
            .await;

    let row = match row {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("DB error on login: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                crate::views::login_page(&form.username, submitted_csrf, Some("Server error.")),
            )
                .into_response();
        }
    };

    let valid = match &row {
        Some(r) => verify_password(&form.password, &r.password_hash),
        None => {
            let _ = verify_password(&form.password, DUMMY_HASH);
            false
        }
    };

    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            crate::views::login_page(
                &form.username,
                submitted_csrf,
                Some("Invalid username or password."),
            ),
        )
            .into_response();
    }

    let user_id = row.unwrap().id;
    let csrf = generate_csrf_token();
    let cookie = make_session_cookie(user_id, &csrf, state.cookie_secure);
    let jar = jar.add(cookie);
    (jar, Redirect::to("/")).into_response()
}

pub async fn handle_post_logout(jar: PrivateCookieJar) -> impl IntoResponse {
    let jar = if let Some(c) = jar.get(cookie_name()) {
        jar.remove(c)
    } else {
        jar
    };
    (jar, Redirect::to("/login"))
}
