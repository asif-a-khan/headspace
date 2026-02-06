use dotenvy::dotenv;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_writer_url: String,
    pub database_reader_url: String,
    pub app_host: String,
    pub app_port: u16,
    pub session_secret: String,
    pub primary_domain: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv().ok();

        Ok(Self {
            database_writer_url: std::env::var("DATABASE_WRITER_URL")?,
            database_reader_url: std::env::var("DATABASE_READER_URL")?,
            app_host: std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            app_port: std::env::var("APP_PORT")
                .unwrap_or_else(|_| "8000".into())
                .parse()?,
            session_secret: std::env::var("SESSION_SECRET")?,
            primary_domain: std::env::var("PRIMARY_DOMAIN")?,
        })
    }
}
