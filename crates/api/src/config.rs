use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use shared::AppResult;

/// Application configuration structure
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub logging: LoggingConfig,
    pub external_services: ExternalServicesConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub default_ttl_seconds: u64,
    pub connection_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExternalServicesConfig {
    pub inventory_service_url: Option<String>,
    pub pricing_service_url: Option<String>,
    pub notification_service_url: Option<String>,
}

impl AppConfig {
    /// Load configuration from environment variables and config files
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            // Start with default values
            .add_source(File::with_name("config/default").required(false))
            // Add environment-specific config (development, production, etc.)
            .add_source(
                File::with_name(&format!("config/{}", 
                    std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string())
                )).required(false)
            )
            // Add local config for overrides
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with prefix "APP"
            .add_source(
                Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__")
            )
            .build()?;

        config.try_deserialize()
    }

    /// Validate configuration values
    pub fn validate(&self) -> AppResult<()> {
        // Server validation
        if self.server.port == 0 {
            return Err(shared::validation_error!("Server port cannot be 0"));
        }

        // Database validation
        if self.database.url.is_empty() {
            return Err(shared::validation_error!("Database URL cannot be empty"));
        }

        if self.database.max_connections == 0 {
            return Err(shared::validation_error!("Database max_connections must be > 0"));
        }

        if self.database.min_connections > self.database.max_connections {
            return Err(shared::validation_error!(
                "Database min_connections cannot exceed max_connections"
            ));
        }

        // Redis validation
        if self.redis.url.is_empty() {
            return Err(shared::validation_error!("Redis URL cannot be empty"));
        }

        if self.redis.pool_size == 0 {
            return Err(shared::validation_error!("Redis pool_size must be > 0"));
        }

        // Logging validation
        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(shared::validation_error!(&format!(
                "Invalid logging level: {}. Valid levels: {:?}",
                self.logging.level, valid_levels
            )));
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
            },
            database: DatabaseConfig {
                url: "postgresql://postgres:password@localhost:5432/products".to_string(),
                max_connections: 10,
                min_connections: 1,
                connection_timeout: 30,
                idle_timeout: 600,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: 10,
                default_ttl_seconds: 3600,
                connection_timeout: 5,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            external_services: ExternalServicesConfig {
                inventory_service_url: None,
                pricing_service_url: None,
                notification_service_url: None,
            },
        }
    }
}

/// Environment-aware configuration loading
impl AppConfig {
    /// Load configuration for development environment
    pub fn development() -> Result<Self, ConfigError> {
        std::env::set_var("APP_ENV", "development");
        Self::load()
    }

    /// Load configuration for production environment
    pub fn production() -> Result<Self, ConfigError> {
        std::env::set_var("APP_ENV", "production");
        Self::load()
    }

    /// Load configuration for testing environment
    pub fn test() -> Result<Self, ConfigError> {
        std::env::set_var("APP_ENV", "test");
        Self::load()
    }

    /// Check if running in development mode
    pub fn is_development(&self) -> bool {
        std::env::var("APP_ENV").unwrap_or_default() == "development"
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        std::env::var("APP_ENV").unwrap_or_default() == "production"
    }

    /// Get database connection pool configuration
    pub fn database_pool_config(&self) -> sqlx::postgres::PgPoolOptions {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(self.database.max_connections)
            .min_connections(self.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(self.database.connection_timeout))
            .idle_timeout(std::time::Duration::from_secs(self.database.idle_timeout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert!(!config.database.url.is_empty());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        
        // Test invalid port
        config.server.port = 0;
        assert!(config.validate().is_err());
        
        // Reset and test invalid database config
        config = AppConfig::default();
        config.database.url = "".to_string();
        assert!(config.validate().is_err());
        
        // Test invalid min/max connections
        config = AppConfig::default();
        config.database.min_connections = 10;
        config.database.max_connections = 5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_environment_detection() {
        std::env::set_var("APP_ENV", "development");
        let config = AppConfig::default();
        assert!(config.is_development());
        assert!(!config.is_production());

        std::env::set_var("APP_ENV", "production");
        let config = AppConfig::default();
        assert!(!config.is_development());
        assert!(config.is_production());
    }
}