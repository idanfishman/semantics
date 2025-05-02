use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::changelog::rule::{CommitSection, Rule};

/// Standard Conventional Commits preset with common categories
/// See: <https://www.conventionalcommits.org/en/v1.0.0/>
pub static CONVENTIONAL_COMMITS_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // Bug Fixes
        Rule::new(
            "Bug Fixes".to_string(),
            r"^fix(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Features
        Rule::new(
            "Features".to_string(),
            r"^feat(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Breaking Changes - from title with !
        Rule::new(
            "Breaking Changes".to_string(),
            r"^\w+!:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Breaking Changes - from body
        Rule::new(
            "Breaking Changes".to_string(),
            r"^BREAKING\sCHANGE:\s.+$",
            Some(CommitSection::Body),
        )
        .unwrap(),
    ]
});

/// Comprehensive Conventional Commits preset with extended categories
/// See: <https://www.conventionalcommits.org/en/v1.0.0/>
pub static CONVENTIONAL_COMMITS_EXTENDED_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // Bug Fixes
        Rule::new(
            "Bug Fixes".to_string(),
            r"^fix(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Features
        Rule::new(
            "Features".to_string(),
            r"^feat(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Breaking Changes - from title with !
        Rule::new(
            "Breaking Changes".to_string(),
            r"^\w+!:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Breaking Changes - from body
        Rule::new(
            "Breaking Changes".to_string(),
            r"^BREAKING\sCHANGE:\s.+$",
            Some(CommitSection::Body),
        )
        .unwrap(),
        // Docs
        Rule::new(
            "Docs".to_string(),
            r"^docs(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Performance Improvements
        Rule::new(
            "Performance Improvements".to_string(),
            r"^perf(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Dependencies
        Rule::new(
            "Dependencies".to_string(),
            r"^deps(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
    ]
});

/// All supported Conventional Commits categories
/// See: <https://www.conventionalcommits.org/en/v1.0.0/>
pub static CONVENTIONAL_COMMITS_ALL_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // Bug Fixes
        Rule::new(
            "Bug Fixes".to_string(),
            r"^fix(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Features
        Rule::new(
            "Features".to_string(),
            r"^feat(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Breaking Changes - from title with !
        Rule::new(
            "Breaking Changes".to_string(),
            r"^\w+!:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Breaking Changes - from body
        Rule::new(
            "Breaking Changes".to_string(),
            r"^BREAKING\sCHANGE:\s.+$",
            Some(CommitSection::Body),
        )
        .unwrap(),
        // Docs
        Rule::new(
            "Docs".to_string(),
            r"^docs(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Code Refactors
        Rule::new(
            "Refactor".to_string(),
            r"^refactor(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Style Changes
        Rule::new(
            "Refactor".to_string(),
            r"^style(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Tests
        Rule::new(
            "Tests".to_string(),
            r"^test(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Build System
        Rule::new(
            "Build System".to_string(),
            r"^build(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // CI
        Rule::new(
            "CI".to_string(),
            r"^ci(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Chore
        Rule::new(
            "Internal".to_string(),
            r"^chore(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Performance Improvements
        Rule::new(
            "Performance Improvements".to_string(),
            r"^perf(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
        // Dependencies
        Rule::new(
            "Dependencies".to_string(),
            r"^deps(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap(),
    ]
});

/// Presets for predefined changelog generator rules.
///
/// The `Preset` enum defines different sets of rules that can be used for categorizing commits in the changelog.
///
/// # Variants
/// * `ConventionalCommits` - A preset with only the standard Conventional Commits categories
/// * `ConventionalCommitsExtended` - A comprehensive preset with extended Conventional Commits categories
/// * `ConventionalCommitsAll` - A preset with all supported Conventional Commits categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    #[serde(rename = "conventional-commits")]
    ConventionalCommits,
    #[serde(rename = "conventional-commits-extended")]
    ConventionalCommitsExtended,
    #[serde(rename = "conventional-commits-all")]
    ConventionalCommitsAll,
}

impl Preset {
    /// Gets the rules associated with this preset.
    ///
    /// # Returns
    ///
    /// Returns a reference to a vector of `Rule` values that make up the preset.
    pub fn get_rules(&self) -> &'static [Rule] {
        match self {
            Self::ConventionalCommits => &CONVENTIONAL_COMMITS_RULES,
            Self::ConventionalCommitsExtended => &CONVENTIONAL_COMMITS_EXTENDED_RULES,
            Self::ConventionalCommitsAll => &CONVENTIONAL_COMMITS_EXTENDED_RULES,
        }
    }
}
