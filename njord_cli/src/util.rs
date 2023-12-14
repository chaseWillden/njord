use core::fmt;
use std::error::Error as StdError;
use std::path::Path;
use std::{env, fs};
use toml::Value as Config;

#[derive(Debug)]
enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "IO error: {}", err),
            ConfigError::Toml(err) => write!(f, "TOML error: {}", err),
        }
    }
}

impl StdError for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::Toml(err)
    }
}

/// Read the configuration from the file.
pub fn read_config() -> Result<Config, ConfigError> {
    let current_dir = env::current_dir()?;

    // construct the path to njord.toml in the root of the repository
    let config_path = current_dir.join("njord.toml");

    // read the content of njord.toml
    let config_content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading config.toml: {}", err);
            return Err(err.into());
        }
    };

    // parse the content
    let config: Config = match toml::from_str(&config_content) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Error parsing config.toml: {}", err);
            return Err(err.into());
        }
    };

    Ok(config)
}

/// Get the next migration version based on the existing ones in the migrations directory.
pub fn get_next_migration_version(migrations_dir: &Path) -> Result<String, std::io::Error> {
    let entries = fs::read_dir(migrations_dir)?;
    let max_version = entries
        .filter_map(|entry| {
            entry
                .ok()
                .and_then(|e| e.file_name().to_str().map(String::from))
        })
        .filter(|version| version.len() == 14)
        .max();

    match max_version {
        Some(max_version) => {
            let next_version: u64 = max_version.parse()?;
            Ok(format!("{:014}", next_version + 1))
        }
        None => Ok("00000000000001".to_string()), // initial version
    }
}

/// Create migration files in the specified directory.
pub fn create_migration_files(
    migrations_dir: &Path,
    version: &str,
    name: &str,
) -> Result<(), std::io::Error> {
    let dir_path = migrations_dir.join(version).join(name);

    fs::create_dir_all(&dir_path)?;

    let up_sql_path = dir_path.join("up.sql");
    let down_sql_path = dir_path.join("down.sql");

    fs::File::create(up_sql_path)?;
    fs::File::create(down_sql_path)?;

    Ok(())
}
