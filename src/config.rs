use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub target: TargetConfig,
    #[allow(dead_code)]
    pub masking: MaskingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TargetConfig {
    pub url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MaskingConfig {
    #[allow(dead_code)]
    pub exclude_fields: Vec<String>,
    #[allow(dead_code)]
    pub max_depth: u8,
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    InvalidConfig(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "Config file not found: {}", path),
            ConfigError::ParseError(msg) => write!(f, "Failed to parse config: {}", msg),
            ConfigError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Path::new("config.yaml");

        // 1. Load from YAML (if exists)
        let mut config: AppConfig = if config_path.exists() {
            let config_str = fs::read_to_string(config_path)
                .map_err(|e| ConfigError::FileNotFound(format!("config.yaml: {}", e)))?;
            serde_yaml::from_str(&config_str)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?
        } else {
             // Fallback default if no config file (useful for pure Docker/Env usage)
             AppConfig {
                 server: ServerConfig { port: 3000, host: "0.0.0.0".to_string() },
                 target: TargetConfig { url: "http://localhost:8080".to_string(), timeout_ms: 5000 },
                 masking: MaskingConfig { exclude_fields: vec![], max_depth: 20 },
             }
        };

        // 2. Override with Environment Variables (Cloud Native)
        if let Ok(port) = std::env::var("PORT") {
            if let Ok(p) = port.parse() { config.server.port = p; }
        }
        if let Ok(url) = std::env::var("TARGET_URL") {
             config.target.url = url;
        } else if let Ok(url) = std::env::var("TARGET_LOG_URL") {
             config.target.url = url;
        }
        if let Ok(depth) = std::env::var("MASKING_MAX_DEPTH") {
            if let Ok(d) = depth.parse() { config.masking.max_depth = d; }
        }

        // Validate config
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        // Validate port
        if self.server.port == 0 {
            return Err(ConfigError::InvalidConfig(
                "Server port cannot be 0".to_string(),
            ));
        }

        // Validate target URL
        if self.target.url.is_empty() {
            return Err(ConfigError::InvalidConfig(
                "Target URL cannot be empty".to_string(),
            ));
        }

        if !self.target.url.starts_with("http://") && !self.target.url.starts_with("https://") {
            return Err(ConfigError::InvalidConfig(format!(
                "Target URL must start with http:// or https://: {}",
                self.target.url
            )));
        }

        // Validate timeout
        if self.target.timeout_ms == 0 {
            return Err(ConfigError::InvalidConfig(
                "Target timeout must be greater than 0".to_string(),
            ));
        }

        // Validate max_depth
        if self.masking.max_depth == 0 {
            return Err(ConfigError::InvalidConfig(
                "Max depth must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_port_zero() {
        let config = AppConfig {
            server: ServerConfig {
                port: 0,
                host: "0.0.0.0".to_string(),
            },
            target: TargetConfig {
                url: "http://localhost:8080".to_string(),
                timeout_ms: 5000,
            },
            masking: MaskingConfig {
                exclude_fields: vec![],
                max_depth: 20,
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_target_url() {
        let config = AppConfig {
            server: ServerConfig {
                port: 3000,
                host: "0.0.0.0".to_string(),
            },
            target: TargetConfig {
                url: "".to_string(),
                timeout_ms: 5000,
            },
            masking: MaskingConfig {
                exclude_fields: vec![],
                max_depth: 20,
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_target_url() {
        let config = AppConfig {
            server: ServerConfig {
                port: 3000,
                host: "0.0.0.0".to_string(),
            },
            target: TargetConfig {
                url: "localhost:8080".to_string(),
                timeout_ms: 5000,
            },
            masking: MaskingConfig {
                exclude_fields: vec![],
                max_depth: 20,
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_zero_timeout() {
        let config = AppConfig {
            server: ServerConfig {
                port: 3000,
                host: "0.0.0.0".to_string(),
            },
            target: TargetConfig {
                url: "http://localhost:8080".to_string(),
                timeout_ms: 0,
            },
            masking: MaskingConfig {
                exclude_fields: vec![],
                max_depth: 20,
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_zero_max_depth() {
        let config = AppConfig {
            server: ServerConfig {
                port: 3000,
                host: "0.0.0.0".to_string(),
            },
            target: TargetConfig {
                url: "http://localhost:8080".to_string(),
                timeout_ms: 5000,
            },
            masking: MaskingConfig {
                exclude_fields: vec![],
                max_depth: 0,
            },
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_valid_config() {
        let config = AppConfig {
            server: ServerConfig {
                port: 3000,
                host: "0.0.0.0".to_string(),
            },
            target: TargetConfig {
                url: "http://localhost:8080".to_string(),
                timeout_ms: 5000,
            },
            masking: MaskingConfig {
                exclude_fields: vec![],
                max_depth: 20,
            },
        };

        assert!(config.validate().is_ok());
    }
}
