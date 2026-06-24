use chrono::NaiveDateTime;
use maud::{html, Markup, DOCTYPE};

const STYLE: &str = "
    body { font-family: system-ui, sans-serif; max-width: 720px; margin: 2rem auto; padding: 0 1rem; line-height: 1.6; }
    a { color: #0066cc; }
    .msg { border-top: 1px solid #ddd; padding: 0.75rem 0; }
    .msg-meta { color: #666; font-size: 0.85em; margin-bottom: 0.25rem; }
    .msg-body { white-space: pre-wrap; word-break: break-word; }
    .error { color: #c00; margin-bottom: 0.5rem; }
    textarea { width: 100%; box-sizing: border-box; }
    input, textarea, button { font-size: 1rem; }
    nav { margin-bottom: 1.5rem; }
";

fn layout(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                style { (STYLE) }
            }
            body { (body) }
        }
    }
}

pub struct TopicRow {
    pub slug: String,
    pub title: String,
}

pub fn index_page(topics: &[TopicRow], username: &str) -> Markup {
    layout(
        "Topics",
        html! {
            nav {
                span { "Logged in as " strong { (username) } " · " }
                form method="post" action="/logout" style="display:inline" {
                    button type="submit" { "Log out" }
                }
            }
            h1 { "Topics" }
            @if topics.is_empty() {
                p { "No topics yet." }
            } @else {
                ul {
                    @for t in topics {
                        li { a href={ "/" (t.slug) } { (t.title) } }
                    }
                }
            }
        },
    )
}

pub struct MessageRow {
    pub author: String,
    pub body: String,
    pub created_at: NaiveDateTime,
}

pub fn thread_page(
    slug: &str,
    title: &str,
    messages: &[MessageRow],
    csrf: &str,
    error: Option<&str>,
    prefill: &str,
) -> Markup {
    layout(
        title,
        html! {
            nav { a href="/" { "← Topics" } }
            h1 { (title) }
            @for m in messages {
                div.msg {
                    div.msg-meta { (m.author) " · " (m.created_at.format("%Y-%m-%d %H:%M UTC")) }
                    div.msg-body { (m.body) }
                }
            }
            @if messages.is_empty() {
                p { "No messages yet. Be the first!" }
            }
            h2 { "Post a message" }
            @if let Some(err) = error {
                p.error { (err) }
            }
            form method="post" action={ "/" (slug) } {
                input type="hidden" name="_csrf" value=(csrf);
                div { textarea name="body" rows="5" { (prefill) } }
                div { button type="submit" { "Post" } }
            }
        },
    )
}

pub fn login_page(username_prefill: &str, csrf: &str, error: Option<&str>) -> Markup {
    layout(
        "Log in",
        html! {
            h1 { "Log in" }
            @if let Some(err) = error {
                p.error { (err) }
            }
            form method="post" action="/login" {
                input type="hidden" name="_csrf" value=(csrf);
                div {
                    label { "Username" br;
                        input type="text" name="username" value=(username_prefill) autocomplete="username" required;
                    }
                }
                div {
                    label { "Password" br;
                        input type="password" name="password" autocomplete="current-password" required;
                    }
                }
                div { button type="submit" { "Log in" } }
            }
        },
    )
}
