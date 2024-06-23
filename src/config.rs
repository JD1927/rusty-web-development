use clap::Parser;
use std::env;

/// Q&A web service API
#[derive(Parser, Debug, PartialEq)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Which errors we want to log (info, warn or error)
    #[clap(short, long, default_value = "info")]
    pub log_level: String,
    /// Web server port
    #[clap(long, default_value = "8080")]
    pub port: u16,
    /// Database user
    #[clap(long, default_value = "postgres")]
    pub db_user: String,
    /// Database password
    #[clap(long, default_value = "postgres")]
    pub db_password: String,
    /// URL for the postgres database
    #[clap(long, default_value = "localhost")]
    pub db_host: String,
    /// PORT number for the database connection
    #[clap(long, default_value = "5432")]
    pub db_port: u16,
    /// Database name
    #[clap(long, default_value = "rustywebdev")]
    pub db_name: String,
}

impl Config {
    pub fn new() -> Result<Config, handle_errors::Error> {
        let config = Config::parse();
        if env::var("BAD_WORDS_API_KEY").is_err() {
            panic!("Bad words API key not set!");
        }

        if env::var("PASETO_KEY").is_err() {
            panic!("PASETO key not set!");
        }
        let port = env::var("PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(config.port))
            .map_err(handle_errors::Error::ParseInt)
            .unwrap();
        let db_user =
            env::var("POSTGRES_USER").unwrap_or(config.db_user.to_owned());
        let db_password = env::var("POSTGRES_PASSWORD").unwrap();
        let db_host =
            env::var("POSTGRES_HOST").unwrap_or(config.db_host.to_owned());
        let db_port = env::var("POSTGRES_PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(config.port))
            .map_err(handle_errors::Error::ParseInt)
            .unwrap();
        let db_name =
            env::var("POSTGRES_DB").unwrap_or(config.db_name.to_owned());

        Ok(Config {
            log_level: config.log_level,
            port,
            db_user,
            db_password,
            db_host,
            db_port,
            db_name,
        })
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    fn set_env() {
        env::set_var("BAD_WORDS_API_KEY", "yes");
        env::set_var("PASETO_KEY", "yes");
        env::set_var("POSTGRES_USER", "user");
        env::set_var("POSTGRES_PASSWORD", "password");
        env::set_var("POSTGRES_DB", "db");
        env::set_var("POSTGRES_HOST", "localhost");
        env::set_var("POSTGRES_PORT", "5432");
    }

    #[test]
    fn unset_and_set_api_key() {
        // UNSET API KEY
        // Arrange
        // Act
        let result = std::panic::catch_unwind(Config::new);
        // Assert
        assert!(result.is_err());

        // SET API KEY
        // Arrange
        set_env();
        let expected = Config {
            log_level: "info".to_string(),
            port: 8080,
            db_user: "user".to_string(),
            db_password: "password".to_string(),
            db_host: "localhost".to_string(),
            db_port: 5432,
            db_name: "db".to_string(),
        };
        // Act
        let result = Config::new().unwrap();
        // Assert
        assert_eq!(result, expected);
    }
}
