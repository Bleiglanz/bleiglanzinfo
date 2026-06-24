use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::io::{self, BufRead};

fn main() {
    let password = if let Some(arg) = std::env::args().nth(1) {
        arg
    } else {
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).expect("read_line failed");
        line.trim().to_string()
    };

    if password.is_empty() {
        eprintln!("Usage: hash <password>  or  echo <password> | hash");
        std::process::exit(1);
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("hashing failed")
        .to_string();

    println!("{hash}");
}
