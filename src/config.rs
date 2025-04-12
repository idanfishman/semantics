use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Clone, Serialize, Deserialize)]
pub struct Config {
    #[validate(custom(function = "validate_tag_format"))]
    tag_format: String,
}

impl Config {
    pub fn new() -> Self {
        Config::default()
    }

    pub fn tag_format(&self) -> &str {
        &self.tag_format
    }

    /// Writes the config to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file where the config should be saved.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or if the config is invalid.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "failed to serialize config to JSON")?;

        let path = path.as_ref();
        fs::write(path, content)
            .with_context(|| format!("failed to write config file to {:?}", path))?;

        Ok(())
    }

    /// Reads the config from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file from which the config should be read.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the config is invalid.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("failed to read config file from {:?}", path.as_ref()))?;

        let config: Config = serde_json::from_str(&content)
            .with_context(|| "failed to deseriazling config from JSON")?;

        config.validate()?;

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            tag_format: String::from("v{version}"),
        }
    }
}

/// Validates that a tag format contains the `{version}` placeholder.
fn validate_tag_format(tag_format: &str) -> Result<(), ValidationError> {
    if !tag_format.contains("{version}") {
        let mut error = ValidationError::new("tag_format");
        error.message = Some(format!("must contain '{{version}}', got '{}'", tag_format).into());
        return Err(error);
    }
    Ok(())
}
