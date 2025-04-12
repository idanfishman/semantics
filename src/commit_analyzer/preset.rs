use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::commit_analyzer::rule::{CommitSection, Rule};
use crate::utils::VersionBump;

/// Predefined rules for analyzing commit messages based on the Conventional Commits specification.
/// See: <https://www.conventionalcommits.org/en/v1.0.0/>
pub static CONVENTIONAL_COMMITS_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        Rule::new(
            VersionBump::Major,
            r"^BREAKING\sCHANGE:\s.+$",
            Some(CommitSection::Body),
        )
        .unwrap(),
        Rule::new(
            VersionBump::Major,
            r"^\w+!:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        Rule::new(
            VersionBump::Minor,
            r"^feat(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        Rule::new(
            VersionBump::Patch,
            r"^fix(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
    ]
});

/// Presets for predefined commit analysis rules.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Preset {
    /// Follows the Conventional Commits specification.
    /// See: <https://www.conventionalcommits.org/en/v1.0.0/>
    #[serde(rename = "conventional-commits")]
    ConventionalCommits,
}
