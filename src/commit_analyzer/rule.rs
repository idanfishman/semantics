use anyhow::Result;
use git2::Commit;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::utils::VersionBump;

/// Specifies the section of a commit message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum CommitSection {
    /// The commit message title.
    #[serde(rename = "title")]
    Title,
    /// The commit message body.
    #[serde(rename = "body")]
    Body,
}

/// Represents a rule for analyzing commit messages.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Regex pattern to match against commit messages.
    pub pattern: Regex,
    /// The regex pattern as a string.
    pub pattern_str: String,
    /// The version bump type associated with the rule.
    pub version_bump: VersionBump,
    /// The commit section to which the rule applies. If `None`, the rule applies to the whole message.
    pub scope: Option<CommitSection>,
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
        if pattern.is_empty() {
            return Err(anyhow::anyhow!("pattern must not be empty"));
        }

        let pattern_re = Regex::new(pattern)?;
        Ok(Rule {
            pattern: pattern_re,
            pattern_str: pattern.to_string(),
            version_bump,
            scope,
        })
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
        let text = match self.scope {
            Some(scope) => match scope {
                CommitSection::Title => commit.summary().unwrap_or("").to_string(),
                CommitSection::Body => commit.body().unwrap_or("").to_string(),
            },
            None => commit.message().unwrap_or("").to_string(),
        };

        if text.is_empty() {
            return None;
        }

        if self.pattern.is_match(&text) {
            Some(self.version_bump)
        } else {
            None
        }
    }
}

impl Serialize for Rule {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct RuleSerialize<'a> {
            #[serde(rename = "pattern")]
            pattern_str: &'a str,
            version_bump: &'a VersionBump,
            #[serde(skip_serializing_if = "Option::is_none")]
            scope: &'a Option<CommitSection>,
        }

        let rule = RuleSerialize {
            pattern_str: &self.pattern_str,
            version_bump: &self.version_bump,
            scope: &self.scope,
        };

        rule.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Rule {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RuleDeserialize {
            #[serde(rename = "pattern")]
            pattern_str: String,
            version_bump: VersionBump,
            scope: Option<CommitSection>,
        }

        let rule = RuleDeserialize::deserialize(deserializer).map_err(|_| serde::de::Error::custom(
            "expected a commit analyzer rule. with 'pattern', 'version_bump' and optionaly 'scope'",
        ))?;

        Rule::new(rule.version_bump, &rule.pattern_str, rule.scope)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use serde_json;

    use crate::commit_analyzer::rule::{CommitSection, Rule};
    use crate::test_helpers::{create_test_commit, create_test_repo};
    use crate::utils::VersionBump;

    #[test]
    fn test_rule_creation_valid() {
        let rule = Rule::new(VersionBump::Patch, r"^fix: .+$", None);
        assert!(rule.is_ok());
    }

    #[test]
    fn test_rule_creation_empty_pattern() {
        let rule = Rule::new(VersionBump::Patch, "", None);
        assert!(rule.is_err());
        assert_eq!(rule.unwrap_err().to_string(), "pattern must not be empty");
    }

    #[test]
    fn test_rule_creation_invalid_regex() {
        let rule = Rule::new(VersionBump::Patch, r"(", None); // Invalid regex pattern
        assert!(rule.is_err());
    }

    #[test]
    fn test_rule_serialization() {
        let rule = Rule::new(VersionBump::Patch, r"^fix: .+$", Some(CommitSection::Title)).unwrap();
        let serialized = serde_json::to_string(&rule).unwrap();
        let expected = r#"{"pattern":"^fix: .+$","version_bump":"patch","scope":"title"}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_rule_deserialization_valid() {
        let json = r#"{"pattern":"^fix: .+$","version_bump":"patch","scope":"title"}"#;
        let rule: Rule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.pattern_str, "^fix: .+$");
        assert_eq!(rule.version_bump, VersionBump::Patch);
        assert_eq!(rule.scope, Some(CommitSection::Title));
    }

    #[test]
    fn test_rule_deserialization_invalid() {
        let json = r#"{"pattern":"^fix: .+$","version_bump":"patch","scope":"invalid"}"#;
        let rule: Result<Rule, _> = serde_json::from_str(json);
        assert!(rule.is_err());
        assert_eq!(
            rule.unwrap_err().to_string(),
            "expected a commit analyzer rule. with 'pattern', 'version_bump' and optionaly 'scope'"
        );
    }

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
