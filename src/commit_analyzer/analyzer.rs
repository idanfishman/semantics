use std::cmp::Reverse;

use anyhow::Result;
use git2::Commit;

use crate::commit_analyzer::config::CommitAnalyzerConfig;
use crate::commit_analyzer::preset::{CONVENTIONAL_COMMITS_RULES, Preset};
use crate::commit_analyzer::rule::Rule;
use crate::semver::VersionBump;

/// Analyzes commit messages to determine the appropriate version bump based on a list of rules.
///
/// The `Analyzer` struct is responsible for evaluating commit messages against a set of predefined
/// or custom rules to determine the type of version bump (e.g., major, minor, patch).
#[derive(Debug)]
pub struct Analyzer {
    pub rules: Vec<Rule>,
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
    ///
    /// # Examples
    /// ```rust
    /// use crate::commit_analyzer::analyzer::Analyzer;
    /// use crate::commit_analyzer::preset::Preset;
    ///
    /// let analyzer = Analyzer::new(Some(Preset::ConventionalCommits), None).unwrap();
    /// assert_eq!(analyzer.rules.len(), 4); // 4 rules in Conventional Commits
    /// ```
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
        all_rules.sort_by_key(|a| Reverse(a.version_bump));

        Ok(Analyzer { rules: all_rules })
    }

    /// Creates a new `Analyzer` instance from the provided `CommitAnalyzerConfig`.
    ///
    /// # Arguments
    ///
    /// * `config` - A reference to the `CommitAnalyzerConfig` containing the preset and/or rules.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Analyzer` instance or an error message.
    ///
    /// # Examples
    /// ```rust
    /// use crate::commit_analyzer::analyzer::Analyzer;
    /// use crate::commit_analyzer::config::CommitAnalyzerConfig;
    /// use crate::commit_analyzer::preset::Preset;
    ///
    /// let config = CommitAnalyzerConfig {
    ///     preset: Some(Preset::ConventionalCommits),
    ///     rules: None,
    /// };
    /// let analyzer = Analyzer::from_config(&config).unwrap();
    /// assert_eq!(analyzer.rules.len(), 4);
    /// ```
    pub fn from_config(config: &CommitAnalyzerConfig) -> Result<Self> {
        Self::new(
            config.preset.as_ref().cloned(),
            config.rules.as_ref().cloned(),
        )
    }

    /// Returns a reference to the rules associated with the given preset.
    ///
    /// # Arguments
    ///
    /// * `preset` - The preset for which to retrieve the rules.
    ///
    /// # Returns
    ///
    /// A reference to the static rules for the given preset.
    ///
    /// # Examples
    /// ```rust
    /// use crate::commit_analyzer::analyzer::Analyzer;
    /// use crate::commit_analyzer::preset::Preset;
    ///
    /// let rules = Analyzer::get_preset_rules(Preset::ConventionalCommits);
    /// assert_eq!(rules.len(), 4);
    /// ```
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
    /// The version bump type based on the commit message, or `None` if no rules match.
    ///
    /// # Examples
    /// ```rust
    /// use crate::commit_analyzer::analyzer::Analyzer;
    /// use crate::commit_analyzer::preset::Preset;
    /// use crate::test_helpers::{create_test_commit, create_test_repo};
    ///
    /// let (repo, _temp_dir) = create_test_repo();
    /// let analyzer = Analyzer::new(Some(Preset::ConventionalCommits), None).unwrap();
    /// let commit = create_test_commit(&repo, "master", "fix: a fix commit");
    /// let result = analyzer.analyze_commit(&commit);
    /// assert_eq!(result, Some(VersionBump::Patch));
    /// ```
    fn analyze_commit(&self, commit: &Commit) -> Option<VersionBump> {
        self.rules.iter().find_map(|rule| rule.eval(commit))
    }

    /// Analyzes a slice of commits and returns the maximum version bump.
    ///
    /// # Arguments
    ///
    /// * `commits` - A slice of `Commit` objects to analyze.
    ///
    /// # Returns
    ///
    /// The maximum version bump type based on the commit messages, or `None` if no rules match.
    ///
    /// # Examples
    /// ```rust
    /// use crate::commit_analyzer::analyzer::Analyzer;
    /// use crate::commit_analyzer::preset::Preset;
    /// use crate::test_helpers::{create_test_commit, create_test_repo};
    ///
    /// let (repo, _temp_dir) = create_test_repo();
    /// let analyzer = Analyzer::new(Some(Preset::ConventionalCommits), None).unwrap();
    /// let commit1 = create_test_commit(&repo, "master", "fix: a fix commit");
    /// let commit2 = create_test_commit(&repo, "master", "feat: a new feature");
    /// let commits = vec![commit1, commit2];
    /// let result = analyzer.analyze_commits(&commits);
    /// assert_eq!(result, Some(VersionBump::Minor));
    /// ```
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
    use crate::commit_analyzer::config::CommitAnalyzerConfig;
    use crate::commit_analyzer::rule::Rule;
    use crate::semver::VersionBump;
    use crate::test_helpers::{create_test_commit, create_test_repo};

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

    #[test]
    fn test_analyzer_from_config_with_preset() {
        let config = CommitAnalyzerConfig {
            preset: Some(Preset::ConventionalCommits),
            rules: None,
        };
        let analyzer = Analyzer::from_config(&config);
        assert!(analyzer.is_ok());
        assert_eq!(analyzer.unwrap().rules.len(), 4); // 4 rules in Conventional Commits
    }

    #[test]
    fn test_analyzer_from_config_with_rules() {
        let rule = Rule::new(VersionBump::Patch, r"^fix: .+$", None).unwrap();
        let config = CommitAnalyzerConfig {
            preset: None,
            rules: Some(vec![rule]),
        };
        let analyzer = Analyzer::from_config(&config);
        assert!(analyzer.is_ok());
        assert_eq!(analyzer.unwrap().rules.len(), 1); // 1 custom rule
    }

    #[test]
    fn test_analyzer_from_config_with_both() {
        let rule = Rule::new(VersionBump::Patch, r"^fix: .+$", None).unwrap();
        let config = CommitAnalyzerConfig {
            preset: Some(Preset::ConventionalCommits),
            rules: Some(vec![rule]),
        };
        let analyzer = Analyzer::from_config(&config);
        assert!(analyzer.is_ok());
        assert_eq!(analyzer.unwrap().rules.len(), 5); // 4 from preset + 1 custom rule
    }

    #[test]
    fn test_analyzer_from_config_with_none() {
        let config = CommitAnalyzerConfig {
            preset: None,
            rules: None,
        };
        let result = Analyzer::from_config(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "no rules provided");
    }
}
