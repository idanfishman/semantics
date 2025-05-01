use serde::{Deserialize, Serialize};

use crate::commit_analyzer::preset::Preset;
use crate::commit_analyzer::rule::Rule;

/// Configuration for the commit analyzer.
///
/// Defines the configuration options for the commit analyzer.
/// It allows specifying a preset of rules or custom rules for analyzing commit messages.
///
/// # Default
/// By default, the configuration uses the `ConventionalCommits` preset and no custom rules.
///
/// # Examples
/// ```rust
/// use crate::commit_analyzer::config::CommitAnalyzerConfig;
/// use crate::commit_analyzer::preset::Preset;
///
/// // Default configuration
/// let default_config = CommitAnalyzerConfig::default();
/// assert_eq!(default_config.preset, Some(Preset::ConventionalCommits));
/// assert!(default_config.rules.is_none());
///
/// // Custom configuration
/// let custom_config = CommitAnalyzerConfig {
///     preset: None,
///     rules: Some(vec![]),
/// };
/// assert!(custom_config.preset.is_none());
/// assert!(custom_config.rules.is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitAnalyzerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<Preset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
}

impl Default for CommitAnalyzerConfig {
    fn default() -> Self {
        CommitAnalyzerConfig {
            preset: Some(Preset::ConventionalCommits),
            rules: None,
        }
    }
}
