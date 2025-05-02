use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::changelog::config::ChangelogGeneratorConfig;
use crate::commit_analyzer::config::CommitAnalyzerConfig;
use crate::release_channel::Channel;

/// Represents the configuration for the semantics tool.
///
/// This struct defines the overall configuration for the tool, including settings for
/// commit analysis, release channels, and tag formatting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Config {
    /// Configuration for analyzing commit messages.
    pub commit_analyzer_config: CommitAnalyzerConfig,

    /// Configuration for generating changelog.
    pub changelog_generator_config: ChangelogGeneratorConfig,

    /// Configuration for generating release notes.
    pub release_notes_generator_config: ChangelogGeneratorConfig,

    /// A list of release channels with their respective settings.
    #[validate(custom(function = "validate_release_channels"))]
    #[validate(nested)]
    pub release_channels: Vec<Channel>,

    /// The format for version tags.
    #[validate(custom(function = "validate_tag_format"))]
    pub tag_format: String,
}

impl Config {
    /// Writes the config to a file.
    ///
    /// # Arguments
    /// * `path` - The path to the file where the config should be saved.
    ///
    /// # Errors
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
    /// * `path` - The path to the file from which the config should be read.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or if the config is invalid.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("failed to read config file from {:?}", path.as_ref()))?;

        let config: Config = serde_json::from_str(&content)
            .with_context(|| "failed to deserialize config from JSON")?;

        config.validate()?;

        Ok(config)
    }
}

impl Default for Config {
    /// Provides a default configuration for the semantics tool.
    ///
    /// The default configuration includes:
    /// * A tag format of `v{version}`.
    /// * Default commit analyzer settings.
    /// * Default changelog generator settings.
    /// * Two release channels: `stable` (non-prerelease) and `rc` (prerelease).
    fn default() -> Self {
        Config {
            tag_format: String::from("v{version}"),

            commit_analyzer_config: CommitAnalyzerConfig::default(),

            changelog_generator_config: ChangelogGeneratorConfig::default(),

            release_notes_generator_config: ChangelogGeneratorConfig::default(),

            release_channels: vec![
                Channel::new("stable", "main", false).unwrap(),
                Channel::new("rc", "main", true).unwrap(),
            ],
        }
    }
}

/// Ensure that tag format ends with the `{version}` placeholder.
///
/// # Arguments
///
/// * `tag_format` - The tag format string to validate.
///
/// # Errors
///
/// Returns a `ValidationError` if the tag format does not end with `{version}`.
fn validate_tag_format(tag_format: &str) -> Result<(), ValidationError> {
    if !tag_format.ends_with("{version}") {
        let mut error = ValidationError::new("tag_format");
        error.message = Some(format!("must end with '{{version}}', got '{}'", tag_format).into());
        return Err(error);
    }
    Ok(())
}

/// Ensures that the release channels list adheres to the following rules:
/// 1. Must contain at least one release channel.
/// 2. No duplicate release channel names.
/// 3. At least one release channel must be marked as not a prerelease.
/// 4. Only one release channel can be marked as not a prerelease.
///
/// # Arguments
///
/// * `release_channels` - A reference to the list of release channels to validate.
///
/// # Errors
///
/// Returns a `ValidationError` if any of the rules are violated.
fn validate_release_channels(release_channels: &Vec<Channel>) -> Result<(), ValidationError> {
    if release_channels.is_empty() {
        let mut error = ValidationError::new("release_channels");
        error.message = Some("must contain at least one release channel".into());
        return Err(error);
    }

    let mut names = std::collections::HashSet::new();
    let mut non_prerelease_count = 0;

    for channel in release_channels {
        if !names.insert(channel.name.to_string()) {
            let mut error = ValidationError::new("release_channels");
            error.message =
                Some(format!("duplicate release channel name: {}", channel.name).into());
            return Err(error);
        }

        if !channel.prerelease {
            non_prerelease_count += 1;
        }
    }

    if non_prerelease_count == 0 {
        let mut error = ValidationError::new("release_channels");
        error.message =
            Some("at least one release channel must be marked as not a prerelease.".into());
        return Err(error);
    }

    if non_prerelease_count > 1 {
        let mut error = ValidationError::new("release_channels");
        error.message = Some("only one release channel can be marked as not a prerelease.".into());
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;
    use validator::Validate;

    use crate::config::Config;
    use crate::release_channel::Channel;

    #[test]
    fn test_config_default_is_valid() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_tag_format_validation() {
        let mut config = Config::default();
        config.tag_format = "release-{version}".to_string();
        assert!(config.validate().is_ok());
        config.tag_format = "release-1.0".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_release_channels_validation() {
        let mut config = Config::default();
        // Valid default
        assert!(config.validate().is_ok());
        // No release channels
        config.release_channels = vec![];
        assert!(config.validate().is_err());
        // Duplicate names
        config.release_channels = vec![
            Channel::new("stable", "main", false).unwrap(),
            Channel::new("stable", "dev", true).unwrap(),
        ];
        assert!(config.validate().is_err());
        // No stable channel
        config.release_channels = vec![Channel::new("rc", "main", true).unwrap()];
        assert!(config.validate().is_err());
        // More than one stable channel
        config.release_channels = vec![
            Channel::new("stable", "main", false).unwrap(),
            Channel::new("prod", "prod", false).unwrap(),
        ];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_save_and_load_config_file() {
        let config = Config::default();
        let file = NamedTempFile::new().unwrap();
        config.save_to_file(file.path()).unwrap();
        let loaded = Config::from_file(file.path()).unwrap();
        assert_eq!(loaded.tag_format, config.tag_format);
        assert_eq!(loaded.release_channels.len(), config.release_channels.len());
    }
}
