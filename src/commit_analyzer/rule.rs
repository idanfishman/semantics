use crate::utils::VersionBump;
use anyhow::Result;
use git2::Commit;
use regex::Regex;

/// Specifies the section of a commit message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitSection {
    /// The commit message title.
    Title,
    /// The commit message body.
    Body,
}

/// Represents a rule for analyzing commit messages.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Regex pattern to match against commit messages.
    pattern: Regex,
    /// The regex pattern as a string.
    pattern_str: String,
    /// The version bump type associated with the rule.
    version_bump: VersionBump,
    /// The commit section to which the rule applies. If `None`, the rule applies to the whole message.
    scope: Option<CommitSection>,
}

impl Rule {
    /// Creates a new instance of Rule.
    ///
    /// # Arguments
    ///
    /// * `version_bump` - The version bump type associated with the rule.
    /// * `pattern` - The regex pattern to match against commit messages.
    /// * `scope` - The commit section to which the rule applies. If `None`, the rule applies to the whole message.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or cannot be compiled into regex.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::commit_analyzer::Rule;
    /// use crate::utils::VersionBump;
    ///
    /// let rule = Rule::new(
    ///    VersionBump::Patch,
    ///   r"^fix(?:\(([^)]+)\))?:\s.+$",
    ///   Some(CommitSection::Title),
    /// ).unwrap();
    /// ```
    pub fn new(
        version_bump: VersionBump,
        pattern: &str,
        scope: Option<CommitSection>,
    ) -> Result<Self> {
        let pattern_re = Regex::new(pattern)?;
        Ok(Rule {
            pattern: pattern_re,
            pattern_str: pattern.to_string(),
            version_bump,
            scope,
        })
    }

    /// Returns a reference to the regex pattern.
    pub fn pattern(&self) -> &Regex {
        &self.pattern
    }

    /// Returns the regex pattern as a string.
    pub fn pattern_str(&self) -> &str {
        &self.pattern_str
    }

    /// Returns the version bump type.
    pub fn version_bump(&self) -> VersionBump {
        self.version_bump
    }

    /// Returns the scope of the rule.
    pub fn scope(&self) -> Option<CommitSection> {
        self.scope
    }

    /// Evaluates the rule against a given commit.
    ///
    /// # Arguments
    ///
    /// * `commit` - A reference to a git commit object.
    ///
    /// # Returns
    ///
    /// * `Some(VersionBump)` if the commit message matches the rule's pattern, otherwise `None`.
    pub fn eval(&self, commit: &Commit) -> Option<VersionBump> {
        let text = match self.scope() {
            Some(scope) => match scope {
                CommitSection::Title => commit.summary().unwrap_or("").to_string(),
                CommitSection::Body => commit.body().unwrap_or("").to_string(),
            },
            None => commit.message().unwrap_or("").to_string(),
        };

        if text.is_empty() {
            return None;
        }

        if self.pattern().is_match(&text) {
            Some(self.version_bump())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::commit_analyzer::rule::{CommitSection, Rule};
    use crate::test_helpers::{create_test_commit, create_test_repo};
    use crate::utils::VersionBump;

    // Rule creation tests
    #[test]
    fn test_rule_creation_valid() {
        let rule = Rule::new(VersionBump::Patch, r"^fix: .+$", None);
        assert!(rule.is_ok());
    }

    #[test]
    fn test_rule_creation_invalid_regex() {
        let rule = Rule::new(VersionBump::Patch, r"(", None); // Invalid regex pattern
        assert!(rule.is_err());
    }

    // Rule evaluation tests
    #[test]
    fn test_rule_eval_title_match() {
        let (repo, _temp_dir) = create_test_repo();
        let rule = Rule::new(
            VersionBump::Patch,
            r"^fix(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap();
        let commit = create_test_commit(&repo, "master", "fix: a fix commit");
        let result = rule.eval(&commit);
        assert_eq!(result, Some(VersionBump::Patch));
    }

    #[test]
    fn test_rule_eval_body_match() {
        let (repo, _temp_dir) = create_test_repo();
        let rule = Rule::new(
            VersionBump::Major,
            r"^BREAKING\sCHANGE:\s.+$",
            Some(CommitSection::Body),
        )
        .unwrap();
        let commit = create_test_commit(
            &repo,
            "master",
            "feat: a feature commit\n\nBREAKING CHANGE: breaking change",
        );
        let result = rule.eval(&commit);
        assert_eq!(result, Some(VersionBump::Major));
    }

    #[test]
    fn test_rule_eval_no_scope_title_match() {
        let (repo, _temp_dir) = create_test_repo();
        let rule = Rule::new(VersionBump::Minor, r"^feat(?:\(([^)]+)\))?:\s.+$", None).unwrap();
        let commit = create_test_commit(&repo, "master", "feat: a feature commit");
        let result = rule.eval(&commit);
        assert_eq!(result, Some(VersionBump::Minor));
    }

    #[test]
    fn test_rule_eval_no_scope_body_match() {
        let (repo, _temp_dir) = create_test_repo();
        let rule = Rule::new(VersionBump::Patch, r"(?i)\bbump\b", None).unwrap(); // Updated regex to match "bump" case-insensitively
        let commit = create_test_commit(
            &repo,
            "master",
            "fix: a fix commit\n\nThis is the body of the commit containing bump",
        );
        let result = rule.eval(&commit);
        assert_eq!(result, Some(VersionBump::Patch));
    }

    #[test]
    fn test_rule_eval_title_no_match() {
        let (repo, _temp_dir) = create_test_repo();
        let rule = Rule::new(VersionBump::Patch, r"^fix: .+$", Some(CommitSection::Title)).unwrap();
        let commit = create_test_commit(&repo, "master", "chore: a chore commit");
        let result = rule.eval(&commit);
        assert_eq!(result, None);
    }

    #[test]
    fn test_rule_eval_body_no_match() {
        let (repo, _temp_dir) = create_test_repo();
        let rule = Rule::new(
            VersionBump::Major,
            r"^BREAKING\sCHANGE:\s.+$",
            Some(CommitSection::Body),
        )
        .unwrap();
        let commit = create_test_commit(&repo, "master", "feat: a feature commit");
        let result = rule.eval(&commit);
        assert_eq!(result, None);
    }

    #[test]
    fn test_rule_eval_no_scope_no_match() {
        let (repo, _temp_dir) = create_test_repo();
        let rule = Rule::new(VersionBump::Patch, r"^fix: .+$", None).unwrap();
        let commit = create_test_commit(&repo, "master", "chore: a chore commit");
        let result = rule.eval(&commit);
        assert_eq!(result, None);
    }

    #[test]
    fn test_rule_eval_empty_commit_message() {
        let (repo, _temp_dir) = create_test_repo();
        let rule = Rule::new(VersionBump::Patch, r"^fix: .+$", None).unwrap();
        let commit = create_test_commit(&repo, "master", "");
        let result = rule.eval(&commit);
        assert_eq!(result, None);
    }
}
