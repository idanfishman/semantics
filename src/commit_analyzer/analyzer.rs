use std::cmp::Reverse;

use anyhow::Result;
use git2::Commit;
use once_cell::sync::Lazy;

use crate::commit_analyzer::rule::{CommitSection, Rule};
use crate::utils::VersionBump;

/// Predefined rules for analyzing commit messages based on the Conventional Commits specification.
/// See: <https://www.conventionalcommits.org/en/v1.0.0/>
static CONVENTIONAL_COMMITS_RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
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
#[derive(Debug, Clone)]
pub enum Preset {
    /// Follows the Conventional Commits specification.
    /// See: <https://www.conventionalcommits.org/en/v1.0.0/>
    ConventionalCommits,
}

/// Analyzes commit messages to determine the appropriate version bump based on list of rules.
#[derive(Debug)]
pub struct Analyzer {
    rules: Vec<Rule>,
}

impl Analyzer {
    /// Creates a new `Analyzer` instance with the provided preset and rules.
    ///
    /// # Arguments
    ///
    /// * `preset` - An optional `Preset` to use for the analyzer.
    /// * `rules` - An optional vector of `Rule` to use for the analyzer.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Analyzer` instance or an error message.
    ///
    /// # Errors
    ///
    /// Returns an error if neither preset nor rules are provided.
    pub fn new(preset: Option<Preset>, rules: Option<Vec<Rule>>) -> Result<Analyzer> {
        let mut all_rules = match (preset, rules) {
            // If both preset and rules are provided, merge them
            (Some(p), Some(r)) => {
                let mut preset_rules = Self::get_preset_rules(p).clone();
                preset_rules.extend(r);
                preset_rules
            }
            // If only preset is provided, use its rules
            (Some(p), None) => Self::get_preset_rules(p).clone(),
            // If only rules are provided, use them
            (None, Some(r)) => r,
            // If neither preset nor rules are provided, return an error
            (None, None) => {
                anyhow::bail!("no rules provided");
            }
        };

        // Sort rules by version bump in descending order
        // This ensures that more significant version bumps are evaluated first
        all_rules.sort_by_key(|a| Reverse(a.version_bump()));

        Ok(Analyzer { rules: all_rules })
    }

    /// Returns a reference to the rules associated with the analyzer.
    pub fn rules(&self) -> &Vec<Rule> {
        &self.rules
    }

    /// Returns a reference to the rules associated with the given preset.
    ///
    /// # Arguments
    ///
    /// * `preset` - The preset for which to retrieve the rules.
    ///
    /// # Returns
    ///
    /// A reference to the static `Vec<Rule>` for the given preset.
    fn get_preset_rules(preset: Preset) -> &'static Vec<Rule> {
        match preset {
            Preset::ConventionalCommits => &CONVENTIONAL_COMMITS_RULES,
        }
    }

    /// Analyzes a commit message and returns the corresponding version bump.
    ///
    /// # Arguments
    ///
    /// * `commit` - A reference to the `Commit` object to analyze.
    ///
    /// # Returns
    ///
    /// An `Option<VersionBump>` indicating the version bump type based on the commit message.
    fn analyze_commit(&self, commit: &Commit) -> Option<VersionBump> {
        self.rules().iter().find_map(|rule| rule.eval(commit))
    }

    /// Analyzes a slice of commits and returns the maximum version bump.
    ///
    /// # Arguments
    ///
    /// * `commits` - A slice of `Commit` objects to analyze.
    ///
    /// # Returns
    ///
    /// An `Option<VersionBump>` indicating the maximum version bump type based on the commit messages.
    pub fn analyze_commits(&self, commits: &[Commit]) -> Option<VersionBump> {
        let mut max_bump = None;

        for commit in commits {
            if let Some(bump) = self.analyze_commit(commit) {
                // Early return if a major version bump is found
                if bump == VersionBump::Major {
                    return Some(VersionBump::Major);
                }
                max_bump = Some(max_bump.map_or(bump, |current: VersionBump| current.max(bump)));
            }
        }

        max_bump
    }
}

#[cfg(test)]
mod test {
    use crate::commit_analyzer::analyzer::{Analyzer, Preset};
    use crate::commit_analyzer::rule::Rule;
    use crate::test_helpers::{create_test_commit, create_test_repo};
    use crate::utils::VersionBump;

    #[test]
    fn test_analyzer_creation_with_preset() {
        let analyzer = Analyzer::new(Some(Preset::ConventionalCommits), None);
        assert!(analyzer.is_ok());
        assert_eq!(analyzer.unwrap().rules.len(), 4); // 4 rules in Conventional Commits
    }

    #[test]
    fn test_analyzer_creation_with_rules() {
        let rule = Rule::new(VersionBump::Patch, r"^fix: .+$", None).unwrap();
        let analyzer = Analyzer::new(None, Some(vec![rule]));
        assert!(analyzer.is_ok());
        assert_eq!(analyzer.unwrap().rules.len(), 1); // 1 custom rule
    }

    #[test]
    fn test_analyzer_creation_with_both() {
        let rule = Rule::new(VersionBump::Patch, r"^fix: .+$", None).unwrap();
        let analyzer = Analyzer::new(Some(Preset::ConventionalCommits), Some(vec![rule]));
        assert!(analyzer.is_ok());
        assert_eq!(analyzer.unwrap().rules.len(), 5); // 4 from preset + 1 custom rule
    }

    #[test]
    fn test_analyzer_creation_with_none() {
        let result = Analyzer::new(None, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "no rules provided")
    }

    #[test]
    fn test_analyze_commit() {
        let (repo, _temp_dir) = create_test_repo();
        let analyzer = Analyzer::new(Some(Preset::ConventionalCommits), None).unwrap();
        let commit = create_test_commit(&repo, "master", "fix: a fix commit");
        let result = analyzer.analyze_commit(&commit);
        assert_eq!(result, Some(VersionBump::Patch));
    }

    #[test]
    fn test_analyze_commits() {
        let (repo, _temp_dir) = create_test_repo();
        let analyzer = Analyzer::new(Some(Preset::ConventionalCommits), None).unwrap();
        let commit1 = create_test_commit(&repo, "master", "fix: a fix commit");
        let commit2 = create_test_commit(&repo, "master", "feat: a new feature");
        let commits = vec![commit1, commit2];
        let result = analyzer.analyze_commits(&commits);
        assert_eq!(result, Some(VersionBump::Minor));
    }

    #[test]
    fn test_analyze_commits_with_no_version_bump() {
        let (repo, _temp_dir) = create_test_repo();
        let analyzer = Analyzer::new(Some(Preset::ConventionalCommits), None).unwrap();
        let commit = create_test_commit(&repo, "master", "chore: a chore commiwt");
        let commits = vec![commit];
        let result = analyzer.analyze_commits(&commits);
        assert_eq!(result, None);
    }
}
