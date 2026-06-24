/// Seed an initial user and optional topics.
/// Usage:
///   DATABASE_URL=sqlite://forum.db USERNAME=alice PASSWORD=secret \
///   TOPICS="general:General Discussion,meta:Meta" \
///   cargo run --bin seed
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::{env, str::FromStr};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let username = env::var("USERNAME").expect("USERNAME must be set");
    let password = env::var("PASSWORD").expect("PASSWORD must be set");
    let topics_env = env::var("TOPICS").unwrap_or_default();

    let opts = SqliteConnectOptions::from_str(&database_url)
        .unwrap()
        .create_if_missing(true)
        .pragma("foreign_keys", "ON");

    let pool = SqlitePool::connect_with(opts)
        .await
        .expect("DB connect failed");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migration failed");

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("hash failed")
        .to_string();

    sqlx::query("INSERT OR IGNORE INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&username)
        .bind(&hash)
        .execute(&pool)
        .await
        .expect("insert user failed");
    println!("User '{username}' seeded.");

    for entry in topics_env.split(',').filter(|s| !s.is_empty()) {
        let (slug, title) = entry
            .split_once(':')
            .expect("TOPICS format: slug:Title,...");
        sqlx::query("INSERT OR IGNORE INTO topics (slug, title) VALUES (?, ?)")
            .bind(slug)
            .bind(title)
            .execute(&pool)
            .await
            .expect("insert topic failed");
        println!("Topic '{slug}' seeded.");
    }
}
