use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::commit_analyzer::rule::{CommitSection, Rule};
use crate::semver::VersionBump;

/// Follows the Conventional Commits specification.
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
///
/// The `Preset` enum defines different sets of rules that can be used for analyzing commit messages.
///
/// # Variants
/// * `ConventionalCommits` - A preset that adheres to the Conventional Commits specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    #[serde(rename = "conventional-commits")]
    ConventionalCommits,
}
