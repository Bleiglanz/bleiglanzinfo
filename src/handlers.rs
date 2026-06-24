use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::PrivateCookieJar;
use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::{
    auth::{
        generate_csrf_token, get_session_csrf, make_session_cookie, parse_session_cookie, AuthUser,
    },
    error::AppError,
    views::{self, MessageRow, TopicRow},
    AppState,
};

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

fn parse_dt(s: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap_or_default()
}

#[derive(sqlx::FromRow)]
struct TopicQueryRow {
    id: i64,
    slug: String,
    title: String,
}

#[derive(sqlx::FromRow)]
struct MessageQueryRow {
    username: String,
    body: String,
    created_at: String,
}

pub async fn get_index(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query_as::<_, TopicQueryRow>(
        "SELECT id, slug, title FROM topics ORDER BY created_at ASC, id ASC",
    )
    .fetch_all(&state.pool)
    .await?;

    let topics: Vec<TopicRow> = rows
        .into_iter()
        .map(|r| TopicRow {
            slug: r.slug,
            title: r.title,
        })
        .collect();
    Ok(views::index_page(&topics, &user.username))
}

pub async fn get_thread(
    State(state): State<AppState>,
    _user: AuthUser,
    jar: PrivateCookieJar,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    let topic =
        sqlx::query_as::<_, TopicQueryRow>("SELECT id, slug, title FROM topics WHERE slug = ?")
            .bind(&slug)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(AppError::NotFound)?;

    let messages = fetch_messages(&state, topic.id).await?;
    let csrf = get_session_csrf(&jar).unwrap_or_default();
    Ok(views::thread_page(&slug, &topic.title, &messages, &csrf, None, "").into_response())
}

#[derive(Deserialize)]
pub struct PostForm {
    pub body: String,
    pub _csrf: Option<String>,
}

pub async fn post_thread(
    State(state): State<AppState>,
    user: AuthUser,
    jar: PrivateCookieJar,
    Path(slug): Path<String>,
    Form(form): Form<PostForm>,
) -> Result<Response, AppError> {
    let topic =
        sqlx::query_as::<_, TopicQueryRow>("SELECT id, slug, title FROM topics WHERE slug = ?")
            .bind(&slug)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(AppError::NotFound)?;

    let expected = get_session_csrf(&jar).unwrap_or_default();
    let submitted = form._csrf.as_deref().unwrap_or("");
    if !constant_time_eq(&expected, submitted) {
        return Err(AppError::Forbidden);
    }

    let body = form.body.trim().to_string();
    if body.is_empty() {
        return render_with_error(
            &state,
            &slug,
            topic.id,
            &topic.title,
            &jar,
            "Message cannot be empty.",
            &form.body,
        )
        .await;
    }
    if body.len() > state.max_body_bytes as usize {
        return render_with_error(
            &state,
            &slug,
            topic.id,
            &topic.title,
            &jar,
            "Message is too long.",
            &form.body,
        )
        .await;
    }

    sqlx::query("INSERT INTO messages (topic_id, author_id, body) VALUES (?, ?, ?)")
        .bind(topic.id)
        .bind(user.id)
        .bind(&body)
        .execute(&state.pool)
        .await?;

    Ok(Redirect::to(&format!("/{slug}")).into_response())
}

async fn fetch_messages(state: &AppState, topic_id: i64) -> Result<Vec<MessageRow>, AppError> {
    let rows = sqlx::query_as::<_, MessageQueryRow>(
        "SELECT u.username, m.body, m.created_at \
         FROM messages m JOIN users u ON u.id = m.author_id \
         WHERE m.topic_id = ? ORDER BY m.created_at ASC, m.id ASC",
    )
    .bind(topic_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| MessageRow {
            author: r.username,
            body: r.body,
            created_at: parse_dt(&r.created_at),
        })
        .collect())
}

async fn render_with_error(
    state: &AppState,
    slug: &str,
    topic_id: i64,
    title: &str,
    jar: &PrivateCookieJar,
    error: &str,
    prefill: &str,
) -> Result<Response, AppError> {
    let messages = fetch_messages(state, topic_id).await?;
    let csrf = get_session_csrf(jar).unwrap_or_default();
    Ok((
        StatusCode::BAD_REQUEST,
        views::thread_page(slug, title, &messages, &csrf, Some(error), prefill),
    )
        .into_response())
}

pub async fn get_login(State(state): State<AppState>, jar: PrivateCookieJar) -> Response {
    // If already authenticated, redirect to /
    if let Some(cookie) = jar.get("session") {
        if let Some((uid, _)) = parse_session_cookie(cookie.value()) {
            if uid > 0 {
                return Redirect::to("/").into_response();
            }
        }
    }

    let csrf = generate_csrf_token();
    let cookie = make_session_cookie(0, &csrf, state.cookie_secure);
    let jar = jar.add(cookie);
    (jar, views::login_page("", &csrf, None)).into_response()
}
