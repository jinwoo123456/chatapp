fn main() {
    // Try to load .env files at build time and inject as rustc-env for mobile targets
    // This makes DATABASE_URL/SECRET_KEY available via option_env!/env! at runtime.
    // We keep it best-effort: if not found, continue.
    #[allow(unused)]
    {
        // Use dotenvy if available to parse .env files
        let _ = dotenvy::from_filename(".env")
            .or_else(|_| dotenvy::from_filename("../.env"))
            .or_else(|_| dotenvy::from_filename("../../.env"));
        if let Ok(val) = std::env::var("DATABASE_URL") {
            println!("cargo:rustc-env=DATABASE_URL={}", val);
        }
        if let Ok(val) = std::env::var("SECRET_KEY") {
            println!("cargo:rustc-env=SECRET_KEY={}", val);
        }
    }
    tauri_build::build()
}
