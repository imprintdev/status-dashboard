pub struct Config {
    pub database_url: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/status_dashboard".to_string());
        let port = std::env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);
        Self { database_url, port }
    }
}
