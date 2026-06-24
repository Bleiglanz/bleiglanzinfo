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
    title: String,
}

#[derive(sqlx::FromRow)]
struct MessageQueryRow {
    id: i64,
    username: String,
    body: String,
    created_at: String,
}

#[derive(sqlx::FromRow)]
struct IndexTopicRow {
    slug: String,
    title: String,
    msg_count: i64,
    last_at: Option<String>,
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

async fn render_index(
    state: &AppState,
    jar: &PrivateCookieJar,
    username: &str,
    error: Option<&str>,
    status: StatusCode,
) -> Result<Response, AppError> {
    let rows = sqlx::query_as::<_, IndexTopicRow>(
        "SELECT t.slug, t.title, COUNT(m.id) AS msg_count, MAX(m.created_at) AS last_at \
         FROM topics t LEFT JOIN messages m ON m.topic_id = t.id \
         GROUP BY t.id ORDER BY last_at DESC, t.created_at DESC, t.id DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    let topics: Vec<TopicRow> = rows
        .into_iter()
        .map(|r| TopicRow {
            slug: r.slug,
            title: r.title,
            msg_count: r.msg_count,
            last_at: r.last_at.as_deref().map(parse_dt),
        })
        .collect();
    let csrf = get_session_csrf(jar).unwrap_or_default();
    Ok((status, views::index_page(&topics, username, &csrf, error)).into_response())
}

pub async fn get_index(
    State(state): State<AppState>,
    user: AuthUser,
    jar: PrivateCookieJar,
) -> Result<Response, AppError> {
    render_index(&state, &jar, &user.username, None, StatusCode::OK).await
}

#[derive(Deserialize)]
pub struct NewTopicForm {
    pub title: String,
    pub _csrf: Option<String>,
}

pub async fn post_index(
    State(state): State<AppState>,
    user: AuthUser,
    jar: PrivateCookieJar,
    Form(form): Form<NewTopicForm>,
) -> Result<Response, AppError> {
    let expected = get_session_csrf(&jar).unwrap_or_default();
    let submitted = form._csrf.as_deref().unwrap_or("");
    if !constant_time_eq(&expected, submitted) {
        return Err(AppError::Forbidden);
    }

    let title = form.title.trim();
    if title.is_empty() {
        return render_index(
            &state,
            &jar,
            &user.username,
            Some("Topic title cannot be empty."),
            StatusCode::BAD_REQUEST,
        )
        .await;
    }

    let base = slugify(title);
    if base.is_empty() {
        return render_index(
            &state,
            &jar,
            &user.username,
            Some("Topic title must contain letters or numbers."),
            StatusCode::BAD_REQUEST,
        )
        .await;
    }

    // Derive a unique slug, appending -2, -3, … on collision.
    let mut slug = base.clone();
    let mut n = 2;
    loop {
        let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM topics WHERE slug = ?")
            .bind(&slug)
            .fetch_one(&state.pool)
            .await?;
        if exists == 0 {
            break;
        }
        slug = format!("{base}-{n}");
        n += 1;
    }

    sqlx::query("INSERT INTO topics (slug, title) VALUES (?, ?)")
        .bind(&slug)
        .bind(title)
        .execute(&state.pool)
        .await?;

    Ok(Redirect::to(&format!("/{slug}")).into_response())
}

pub async fn get_thread(
    State(state): State<AppState>,
    user: AuthUser,
    jar: PrivateCookieJar,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    let topic = sqlx::query_as::<_, TopicQueryRow>("SELECT id, title FROM topics WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let messages = fetch_messages(&state, topic.id).await?;
    let csrf = get_session_csrf(&jar).unwrap_or_default();
    Ok(views::thread_page(
        &slug,
        &topic.title,
        &messages,
        &user.username,
        &csrf,
        None,
        "",
    )
    .into_response())
}

#[derive(Deserialize)]
pub struct DeleteForm {
    pub msg_id: i64,
    pub _csrf: Option<String>,
}

/// Delete a message, but only if it is the latest in the topic and owned by
/// the requester. The guard runs in SQL so a concurrent reply between page
/// load and submit makes the delete a no-op rather than removing a
/// now-non-last message.
pub async fn delete_message(
    State(state): State<AppState>,
    user: AuthUser,
    jar: PrivateCookieJar,
    Path(slug): Path<String>,
    Form(form): Form<DeleteForm>,
) -> Result<Response, AppError> {
    let topic = sqlx::query_as::<_, TopicQueryRow>("SELECT id, title FROM topics WHERE slug = ?")
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let expected = get_session_csrf(&jar).unwrap_or_default();
    let submitted = form._csrf.as_deref().unwrap_or("");
    if !constant_time_eq(&expected, submitted) {
        return Err(AppError::Forbidden);
    }

    sqlx::query(
        "DELETE FROM messages \
         WHERE id = ?1 AND author_id = ?2 AND topic_id = ?3 \
         AND id = (SELECT MAX(id) FROM messages WHERE topic_id = ?3)",
    )
    .bind(form.msg_id)
    .bind(user.id)
    .bind(topic.id)
    .execute(&state.pool)
    .await?;

    Ok(Redirect::to(&format!("/{slug}")).into_response())
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
    let topic = sqlx::query_as::<_, TopicQueryRow>("SELECT id, title FROM topics WHERE slug = ?")
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
            &user.username,
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
            &user.username,
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
        "SELECT m.id, u.username, m.body, m.created_at \
         FROM messages m JOIN users u ON u.id = m.author_id \
         WHERE m.topic_id = ? ORDER BY m.created_at ASC, m.id ASC",
    )
    .bind(topic_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| MessageRow {
            id: r.id,
            author: r.username,
            body: r.body,
            created_at: parse_dt(&r.created_at),
        })
        .collect())
}

#[allow(clippy::too_many_arguments)]
async fn render_with_error(
    state: &AppState,
    slug: &str,
    topic_id: i64,
    title: &str,
    current_user: &str,
    jar: &PrivateCookieJar,
    error: &str,
    prefill: &str,
) -> Result<Response, AppError> {
    let messages = fetch_messages(state, topic_id).await?;
    let csrf = get_session_csrf(jar).unwrap_or_default();
    Ok((
        StatusCode::BAD_REQUEST,
        views::thread_page(
            slug,
            title,
            &messages,
            current_user,
            &csrf,
            Some(error),
            prefill,
        ),
    )
        .into_response())
}

pub async fn get_login(State(state): State<AppState>, jar: PrivateCookieJar) -> Response {
    if let Some(cookie) = jar.get("session") {
        if let Some((uid, csrf)) = parse_session_cookie(cookie.value()) {
            // Already authenticated -> go home.
            if uid > 0 {
                return Redirect::to("/").into_response();
            }
            // A pre-auth cookie already exists: reuse its CSRF token instead of
            // minting a new one. Otherwise every extra GET /login (favicon
            // redirect, refresh, prefetch) would rotate the cookie and
            // invalidate the token embedded in an already-rendered form.
            if !csrf.is_empty() {
                return views::login_page("", &csrf, None).into_response();
            }
        }
    }

    let csrf = generate_csrf_token();
    let cookie = make_session_cookie(0, &csrf, state.cookie_secure);
    let jar = jar.add(cookie);
    (jar, views::login_page("", &csrf, None)).into_response()
}
