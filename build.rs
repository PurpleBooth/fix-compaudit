use std::env;

fn main() {
    println!(
        "cargo:rustc-env=VERSION={}",
        env::var("VERSION").unwrap_or("dev".to_string())
    );
}
