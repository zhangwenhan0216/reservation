use std::fs;

use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DbConfig {
  pub host: String,
  pub port: u16,
  pub username: String,
  pub password: String,
  pub database: String,
  #[serde(default = "default_max_connections")]
  pub max_connections: u32,
}

fn default_max_connections() -> u32 {
  5
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ServerConfig {
  pub host: String,
  pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Config {
  pub db: DbConfig,
  pub server: ServerConfig,
}

impl Config {
  pub fn from_file(filename: &str) -> Result<Self, Error> {
    let str =
      fs::read_to_string(filename).map_err(|_| Error::ConfigReadError)?;

    Ok(serde_yml::from_str(&str).map_err(|_| Error::ConfigParseError)?)
  }
}
impl DbConfig {
  pub fn server_url(&self) -> String {
    if self.password.is_empty() {
      return format!(
        "postgres://{}@{}:{}",
        self.username, self.host, self.port
      );
    } else {
      format!(
        "postgres://{}:{}@{}:{}",
        self.username, self.password, self.host, self.port
      )
    }
  }
  pub fn get_url(&self) -> String {
    format!("{}/{}", self.server_url(), self.database)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_config_from_file() {
    let config = Config::from_file("../service/fixtures/config.yml").unwrap();
    assert_eq!(
      config,
      Config {
        db: DbConfig {
          host: "localhost".to_string(),
          port: 5432,
          username: "postgres".to_string(),
          password: "123456".to_string(),
          database: "reservation".to_string(),
          max_connections: 5,
        },
        server: ServerConfig {
          host: "0.0.0.0".to_string(),
          port: 50051,
        }
      }
    );
  }
}
