use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::commit_analyzer::rule::{CommitSection, Rule};
use crate::semver::VersionBump;

/// Predefined rules for analyzing commit messages based on the Conventional Commits specification.
///
/// These rules are used to determine the type of version bump (e.g., major, minor, patch) based on
/// the commit message format. The rules follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) standard.
///
/// # Rules
/// * **Major Version Bump**:
///   - Matches commit messages with a `BREAKING CHANGE:` prefix in the body.
///   - Matches commit messages with a `!` after the type in the title (e.g., `feat!:`, `fix!:`).
/// * **Minor Version Bump**:
///   - Matches commit messages with a `feat` type in the title (e.g., `feat: add new feature`).
/// * **Patch Version Bump**:
///   - Matches commit messages with a `fix` type in the title (e.g., `fix: resolve bug`).
///
/// # Example
/// ```rust
/// use crate::commit_analyzer::preset::CONVENTIONAL_COMMITS_RULES;
///
/// for rule in CONVENTIONAL_COMMITS_RULES.iter() {
///     println!("Rule: {:?}", rule);
/// }
/// ```
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
///
/// # Example
/// ```rust
/// use crate::commit_analyzer::preset::Preset;
///
/// let preset = Preset::ConventionalCommits;
/// println!("Using preset: {:?}", preset);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    /// Follows the Conventional Commits specification.
    /// See: <https://www.conventionalcommits.org/en/v1.0.0/>
    #[serde(rename = "conventional-commits")]
    ConventionalCommits,
}
