use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Europe::Berlin;
use maud::{html, Markup, PreEscaped, DOCTYPE};

/// Format a UTC timestamp for display in the Europe/Berlin timezone
/// (CET/CEST, DST-aware). Shows the zone abbreviation, e.g. "CEST".
fn fmt_berlin(utc: NaiveDateTime) -> String {
    Berlin
        .from_utc_datetime(&utc)
        .format("%Y-%m-%d %H:%M %Z")
        .to_string()
}

const STYLE: &str = "
    :root {
        --font-serif: 'Iowan Old Style', 'Palatino Linotype', Palatino, 'Book Antiqua', Georgia, serif;
        --font-sans: ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
        --font-mono: ui-monospace, 'SF Mono', 'JetBrains Mono', Menlo, Consolas, monospace;
        --bg: #f6f3ec;
        --surface: #fffdf8;
        --text: #211f1a;
        --muted: #6c685e;
        --border: #e6e0d3;
        --accent: #1f6f63;
        --accent-strong: #155a50;
        --accent-soft: rgba(31,111,99,0.10);
        --danger: #b3401f;
        --shadow: 0 1px 2px rgba(33,31,26,0.04), 0 6px 20px rgba(33,31,26,0.05);
        --radius: 10px;
    }
    @media (prefers-color-scheme: dark) {
        :root {
            --bg: #15140f;
            --surface: #1d1b15;
            --text: #ece7da;
            --muted: #a39d8d;
            --border: #322e25;
            --accent: #62b6a6;
            --accent-strong: #7cc6b8;
            --accent-soft: rgba(98,182,166,0.12);
            --danger: #e07a59;
            --shadow: 0 1px 2px rgba(0,0,0,0.3), 0 8px 24px rgba(0,0,0,0.28);
        }
    }
    * { box-sizing: border-box; }
    html { -webkit-text-size-adjust: 100%; }
    body {
        font-family: var(--font-sans);
        max-width: 46rem;
        margin: 0 auto;
        padding: 3rem 1.25rem 5rem;
        line-height: 1.65;
        color: var(--text);
        background: var(--bg);
        background-image: radial-gradient(120% 90% at 50% -10%, var(--accent-soft), transparent 60%);
        background-attachment: fixed;
        -webkit-font-smoothing: antialiased;
        font-feature-settings: 'kern', 'liga';
        animation: rise 0.5s cubic-bezier(0.2, 0.7, 0.2, 1) both;
    }
    @keyframes rise { from { opacity: 0; transform: translateY(6px); } to { opacity: 1; transform: none; } }
    @media (prefers-reduced-motion: reduce) { body { animation: none; } }
    h1, h2, h3 { font-family: var(--font-serif); font-weight: 600; line-height: 1.2; letter-spacing: -0.01em; }
    h1 { font-size: 1.95rem; margin: 0 0 1.5rem; }
    h2 { font-size: 1.25rem; margin: 0 0 0.75rem; }
    a { color: var(--accent); text-decoration: none; transition: color 0.15s; }
    a:hover { color: var(--accent-strong); text-decoration: underline; text-underline-offset: 3px; }
    p { margin: 0 0 1rem; }

    nav {
        display: flex; align-items: center; justify-content: space-between; gap: 1rem;
        margin-bottom: 2.25rem; padding-bottom: 1rem;
        border-bottom: 1px solid var(--border);
        font-size: 0.9rem; color: var(--muted);
    }
    nav a { font-weight: 500; }

    input, textarea, button { font-size: 1rem; font-family: inherit; }
    input[type=text], input[type=password], textarea {
        width: 100%; padding: 0.6rem 0.75rem; color: var(--text);
        background: var(--surface); border: 1px solid var(--border);
        border-radius: var(--radius); transition: border-color 0.15s, box-shadow 0.15s;
    }
    input:focus, textarea:focus {
        outline: none; border-color: var(--accent);
        box-shadow: 0 0 0 3px var(--accent-soft);
    }
    textarea { resize: vertical; min-height: 6.5rem; }
    label { display: block; font-size: 0.85rem; font-weight: 600; color: var(--muted); margin-bottom: 1rem; }
    label input { margin-top: 0.35rem; }

    button {
        cursor: pointer; padding: 0.55rem 1.15rem; font-weight: 600;
        color: #fff; background: var(--accent); border: 1px solid transparent;
        border-radius: var(--radius); transition: background 0.15s, transform 0.05s, box-shadow 0.15s;
    }
    button:hover { background: var(--accent-strong); }
    button:active { transform: translateY(1px); }
    button:focus-visible { outline: none; box-shadow: 0 0 0 3px var(--accent-soft); }
    nav button {
        color: var(--muted); background: transparent; border-color: var(--border);
        padding: 0.35rem 0.8rem; font-weight: 500; font-size: 0.85rem;
    }
    nav button:hover { color: var(--text); background: var(--surface); border-color: var(--muted); }

    table { border-collapse: collapse; width: 100%; }
    thead th {
        text-align: left; padding: 0 0.75rem 0.6rem; color: var(--muted);
        font-weight: 600; font-size: 0.72rem; letter-spacing: 0.06em; text-transform: uppercase;
        border-bottom: 1px solid var(--border);
    }
    tbody td { padding: 0.85rem 0.75rem; border-bottom: 1px solid var(--border); vertical-align: baseline; }
    tbody tr { transition: background 0.12s; }
    tbody tr:hover { background: var(--accent-soft); }
    tbody td:first-child a { font-family: var(--font-serif); font-size: 1.08rem; font-weight: 600; }
    td.num, th.num { text-align: right; font-family: var(--font-mono); font-variant-numeric: tabular-nums; color: var(--muted); }
    tbody td:last-child { color: var(--muted); font-size: 0.85rem; white-space: nowrap; }

    .msg {
        background: var(--surface); border: 1px solid var(--border); border-left: 3px solid var(--accent);
        border-radius: var(--radius); padding: 0.9rem 1.1rem; margin-bottom: 0.9rem; box-shadow: var(--shadow);
    }
    .msg-meta { display: flex; align-items: center; gap: 0.5rem; color: var(--muted); font-size: 0.8rem; margin-bottom: 0.35rem; }
    .msg-meta strong, .msg-author { color: var(--text); font-weight: 600; }
    .msg-body { white-space: pre-wrap; word-break: break-word; }
    .msg-actions { margin-left: auto; display: flex; align-items: center; gap: 0.4rem; }
    .msg-btn, .msg-delete button {
        padding: 0.15rem 0.55rem; font-size: 0.72rem; font-weight: 500; line-height: 1.6;
        background: transparent; border: 1px solid var(--border); border-radius: 6px;
        cursor: pointer; text-decoration: none;
    }
    .msg-btn { color: var(--accent); }
    .msg-btn:hover { border-color: var(--accent); background: var(--accent-soft); text-decoration: none; }
    .msg-delete button { color: var(--danger); }
    .msg-delete button:hover { border-color: var(--danger); background: rgba(179,64,31,0.08); }
    .edit-actions { display: flex; align-items: center; gap: 0.9rem; margin-top: 0.75rem; }
    .edit-actions .btn-cancel { font-size: 0.9rem; color: var(--muted); }

    .error {
        color: var(--danger); background: rgba(179,64,31,0.08); border: 1px solid rgba(179,64,31,0.25);
        border-radius: var(--radius); padding: 0.6rem 0.85rem; margin-bottom: 1rem; font-size: 0.9rem;
    }

    .new-topic { margin-top: 2.75rem; padding-top: 1.75rem; border-top: 1px solid var(--border); }
    .new-topic input[name=title] { margin-bottom: 0.75rem; }
    form > div + div { margin-top: 1rem; }

    .card {
        background: var(--surface); border: 1px solid var(--border);
        border-radius: var(--radius); padding: 1.75rem; box-shadow: var(--shadow);
    }

    .editor { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; align-items: stretch; }
    .editor-pane { display: flex; flex-direction: column; }
    .field-label, .editor-pane label { font-size: 0.85rem; font-weight: 600; color: var(--muted); margin-bottom: 0.4rem; }
    .editor textarea { flex: 1; min-height: 11rem; }
    .preview {
        flex: 1; min-height: 11rem; margin: 0; overflow: auto;
        background: var(--surface); border: 1px solid var(--border);
        border-radius: var(--radius); padding: 0.6rem 0.75rem;
    }
    .preview:empty::before { content: 'Nothing to preview yet.'; color: var(--muted); }
    .tex-hint { font-size: 0.8rem; color: var(--muted); margin: 0.6rem 0 1rem; }
    .tex-hint code { font-family: var(--font-mono); background: var(--accent-soft); padding: 0.05rem 0.35rem; border-radius: 5px; font-size: 0.9em; }
    @media (max-width: 640px) { .editor { grid-template-columns: 1fr; } }
";

const KATEX_VERSION: &str = "0.16.11";
const KATEX_CSS_SRI: &str =
    "sha384-nB0miv6/jRmo5UMMR1wu3Gz6NLsoTkbqJghGIsx//Rlm+ZU03BU6SQNC66uf4l5+";
const KATEX_JS_SRI: &str =
    "sha384-7zkQWkzuo3B5mTepMUcHkMB5jZaolc2xDwL6VFqjFALcbeS9Ggm/Yr2r3Dy4lfFg";
const KATEX_AUTORENDER_SRI: &str =
    "sha384-43gviWU0YVjaDtb/GhzOouOXtZMP/7XUzwPTstBeZFe/+rCMvRwr4yROQP43s0Xk";

const KATEX_INIT: &str = r#"
(function () {
    var opts = {
        delimiters: [
            { left: '$$', right: '$$', display: true },
            { left: '$', right: '$', display: false },
            { left: '\\(', right: '\\)', display: false },
            { left: '\\[', right: '\\]', display: true }
        ],
        throwOnError: false
    };
    function render(el) {
        if (window.renderMathInElement) {
            try { renderMathInElement(el, opts); } catch (e) {}
        }
    }
    document.addEventListener('DOMContentLoaded', function () {
        document.querySelectorAll('.msg-body:not(.preview)').forEach(render);
        document.querySelectorAll('.editor').forEach(function (ed) {
            var ta = ed.querySelector('textarea');
            var pv = ed.querySelector('.preview');
            if (ta && pv) {
                var upd = function () { pv.textContent = ta.value; render(pv); };
                ta.addEventListener('input', upd);
                upd();
            }
        });
    });
})();
"#;

fn layout(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                link rel="stylesheet"
                    href={ "https://cdn.jsdelivr.net/npm/katex@" (KATEX_VERSION) "/dist/katex.min.css" }
                    integrity=(KATEX_CSS_SRI)
                    crossorigin="anonymous";
                style { (PreEscaped(STYLE)) }
            }
            body {
                (body)
                script defer
                    src={ "https://cdn.jsdelivr.net/npm/katex@" (KATEX_VERSION) "/dist/katex.min.js" }
                    integrity=(KATEX_JS_SRI)
                    crossorigin="anonymous" {}
                script defer
                    src={ "https://cdn.jsdelivr.net/npm/katex@" (KATEX_VERSION) "/dist/contrib/auto-render.min.js" }
                    integrity=(KATEX_AUTORENDER_SRI)
                    crossorigin="anonymous" {}
                script { (PreEscaped(KATEX_INIT)) }
            }
        }
    }
}

pub struct TopicRow {
    pub slug: String,
    pub title: String,
    pub msg_count: i64,
    pub last_at: Option<NaiveDateTime>,
}

pub fn index_page(topics: &[TopicRow], username: &str, csrf: &str, error: Option<&str>) -> Markup {
    layout(
        "Topics",
        html! {
            nav {
                span { "Logged in as " strong { (username) } }
                form method="post" action="/logout" style="display:inline" {
                    button type="submit" { "Log out" }
                }
            }
            h1 { "Topics" }
            @if topics.is_empty() {
                p { "No topics yet." }
            } @else {
                table {
                    thead {
                        tr {
                            th { "Topic" }
                            th.num { "Messages" }
                            th { "Last message" }
                        }
                    }
                    tbody {
                        @for t in topics {
                            tr {
                                td { a href={ "/" (t.slug) } { (t.title) } }
                                td.num { (t.msg_count) }
                                td {
                                    @if let Some(dt) = t.last_at {
                                        (fmt_berlin(dt))
                                    } @else {
                                        "—"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div.new-topic {
                h2 { "New topic" }
                @if let Some(err) = error {
                    p.error { (err) }
                }
                form method="post" action="/" {
                    input type="hidden" name="_csrf" value=(csrf);
                    input type="text" name="title" placeholder="Topic title" required;
                    div { button type="submit" { "Create topic" } }
                }
            }
        },
    )
}

pub struct MessageRow {
    pub id: i64,
    pub author: String,
    pub body: String,
    pub created_at: NaiveDateTime,
}

#[allow(clippy::too_many_arguments)]
pub fn thread_page(
    slug: &str,
    title: &str,
    messages: &[MessageRow],
    current_user: &str,
    csrf: &str,
    error: Option<&str>,
    prefill: &str,
    editing: Option<i64>,
) -> Markup {
    let last_index = messages.len().checked_sub(1);
    layout(
        title,
        html! {
            nav { a href="/" { "← Topics" } }
            h1 { (title) }
            @for (i, m) in messages.iter().enumerate() {
                @let is_last_owned = Some(i) == last_index && m.author == current_user;
                @let is_editing = is_last_owned && Some(m.id) == editing;
                div.msg {
                    div.msg-meta {
                        span.msg-author { (m.author) } " · " (fmt_berlin(m.created_at))
                        @if is_last_owned && !is_editing {
                            span.msg-actions {
                                a.msg-btn href={ "/" (slug) "?edit=" (m.id) } { "Edit" }
                                form.msg-delete method="post" action={ "/" (slug) "/delete" } {
                                    input type="hidden" name="_csrf" value=(csrf);
                                    input type="hidden" name="msg_id" value=(m.id);
                                    button type="submit" { "Delete" }
                                }
                            }
                        }
                    }
                    @if is_editing {
                        form method="post" action={ "/" (slug) "/edit" } {
                            input type="hidden" name="_csrf" value=(csrf);
                            input type="hidden" name="msg_id" value=(m.id);
                            div.editor {
                                div.editor-pane {
                                    textarea name="body" rows="6"
                                        placeholder="Edit your message… TeX math is supported." { (m.body) }
                                }
                                div.editor-pane {
                                    span.field-label { "Preview" }
                                    div.msg-body.preview {}
                                }
                            }
                            div.edit-actions {
                                button type="submit" { "Save" }
                                a.btn-cancel href={ "/" (slug) } { "Cancel" }
                            }
                        }
                    } @else {
                        div.msg-body { (m.body) }
                    }
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
                div.editor {
                    div.editor-pane {
                        label for="body" { "Message" }
                        textarea #body name="body" rows="10"
                            placeholder="Write a message… TeX math is supported." { (prefill) }
                    }
                    div.editor-pane {
                        span.field-label { "Preview" }
                        div #preview.msg-body.preview {}
                    }
                }
                p.tex-hint {
                    "Supports TeX — inline " code { "$a^2+b^2$" }
                    " and display " code { "$$ \\sum_{i=1}^n i $$" } "."
                }
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
            div.card {
                @if let Some(err) = error {
                    p.error { (err) }
                }
                form method="post" action="/login" {
                    input type="hidden" name="_csrf" value=(csrf);
                    div {
                        label { "Username"
                            input type="text" name="username" value=(username_prefill) autocomplete="username" required;
                        }
                    }
                    div {
                        label { "Password"
                            input type="password" name="password" autocomplete="current-password" required;
                        }
                    }
                    div { button type="submit" { "Log in" } }
                }
            }
        },
    )
}
