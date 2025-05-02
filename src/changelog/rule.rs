use anyhow::Result;
use git2::Commit;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Represents a category in the changelog.
///
/// Commits will be grouped by category in the generated changelog.
pub type Category = String;

/// Specifies the section of a commit message to match against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitSection {
    /// The commit message title.
    #[serde(rename = "title")]
    Title,
    /// The commit message body.
    #[serde(rename = "body")]
    Body,
}

/// Represents a rule for categorizing commit messages in the changelog.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Regex pattern to match against commit messages.
    pub pattern: Regex,
    /// The regex pattern as a string.
    pub pattern_str: String,
    /// The category associated with the rule.
    pub category: Category,
    /// The commit section to which the rule applies. If `None`, the rule applies to the whole message.
    pub scope: Option<CommitSection>,
}

impl Rule {
    /// Creates a new instance of Rule.
    ///
    /// # Arguments
    ///
    /// * `category` - The category associated with the rule.
    /// * `pattern` - The regex pattern to match against commit messages.
    /// * `scope` - The commit section to which the rule applies. If `None`, the rule applies to the whole message.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern is invalid or cannot be compiled into regex.
    pub fn new(category: Category, pattern: &str, scope: Option<CommitSection>) -> Result<Self> {
        if pattern.is_empty() {
            return Err(anyhow::anyhow!("pattern must not be empty"));
        }

        let pattern_re = Regex::new(pattern)?;
        Ok(Rule {
            pattern: pattern_re,
            pattern_str: pattern.to_string(),
            category,
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
    /// * `true` if the commit message matches the rule's pattern, otherwise `false`.
    pub fn eval(&self, commit: &Commit) -> bool {
        let text = match self.scope {
            Some(scope) => match scope {
                CommitSection::Title => commit.summary().unwrap_or("").to_string(),
                CommitSection::Body => commit.body().unwrap_or("").to_string(),
            },
            None => commit.message().unwrap_or("").to_string(),
        };

        if text.is_empty() {
            return false;
        }

        self.pattern.is_match(&text)
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.pattern_str == other.pattern_str
            && self.category == other.category
            && self.scope == other.scope
    }
}

impl Eq for Rule {}

impl Serialize for Rule {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct RuleSerialize<'a> {
            #[serde(rename = "pattern")]
            pattern_str: &'a str,
            category: &'a Category,
            #[serde(skip_serializing_if = "Option::is_none")]
            scope: &'a Option<CommitSection>,
        }

        let rule = RuleSerialize {
            pattern_str: &self.pattern_str,
            category: &self.category,
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
            category: Category,
            scope: Option<CommitSection>,
        }

        let rule = RuleDeserialize::deserialize(deserializer).map_err(|_| {
            serde::de::Error::custom(
                "expected a changelog rule with 'pattern', 'category', and optionally 'scope'",
            )
        })?;

        Rule::new(rule.category, &rule.pattern_str, rule.scope).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use serde_json;

    use crate::changelog::rule::{CommitSection, Rule};
    use crate::test_helpers::{create_test_commit, create_test_repo};

    #[test]
    fn test_rule_creation_valid() {
        let category = "Features".to_string();
        let rule = Rule::new(category, r"^feat: .+$", None);
        assert!(rule.is_ok());
    }

    #[test]
    fn test_rule_creation_empty_pattern() {
        let category = "Features".to_string();
        let rule = Rule::new(category, "", None);
        assert!(rule.is_err());
        assert_eq!(rule.unwrap_err().to_string(), "pattern must not be empty");
    }

    #[test]
    fn test_rule_creation_invalid_regex() {
        let category = "Features".to_string();
        let rule = Rule::new(category, r"(", None); // Invalid regex pattern
        assert!(rule.is_err());
    }

    #[test]
    fn test_rule_serialization() {
        let category = "Features".to_string();
        let rule = Rule::new(category, r"^feat: .+$", Some(CommitSection::Title)).unwrap();
        let serialized = serde_json::to_string(&rule).unwrap();
        let expected = r#"{"pattern":"^feat: .+$","category":"Features","scope":"title"}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_rule_deserialization_valid() {
        let json = r#"{"pattern":"^feat: .+$","category":"Features","scope":"title"}"#;
        let rule: Rule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.pattern_str, "^feat: .+$");
        assert_eq!(rule.category, "Features");
        assert_eq!(rule.scope, Some(CommitSection::Title));
    }

    #[test]
    fn test_rule_deserialization_invalid() {
        let json = r#"{"pattern":"^feat: .+$","category":"Features","scope":"invalid"}"#;
        let rule: Result<Rule, _> = serde_json::from_str(json);
        assert!(rule.is_err());
        assert!(
            rule.unwrap_err()
                .to_string()
                .contains("expected a changelog rule")
        );
    }

    #[test]
    fn test_rule_eval_title_match() {
        let (repo, _temp_dir) = create_test_repo();
        let category = "Features".to_string();
        let rule = Rule::new(
            category,
            r"^feat(?:\(([^)]+)\))?:\s.+$",
            Some(CommitSection::Title),
        )
        .unwrap();
        let commit = create_test_commit(&repo, "master", "feat: new feature");
        let result = rule.eval(&commit);
        assert!(result);
    }

    #[test]
    fn test_rule_eval_body_match() {
        let (repo, _temp_dir) = create_test_repo();
        let category = "Breaking Changes".to_string();
        let rule = Rule::new(
            category,
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
        assert!(result);
    }

    #[test]
    fn test_rule_eval_no_scope_title_match() {
        let (repo, _temp_dir) = create_test_repo();
        let category = "Features".to_string();
        let rule = Rule::new(category, r"^feat(?:\(([^)]+)\))?:\s.+$", None).unwrap();
        let commit = create_test_commit(&repo, "master", "feat: a feature commit");
        let result = rule.eval(&commit);
        assert!(result);
    }

    #[test]
    fn test_rule_eval_no_scope_body_match() {
        let (repo, _temp_dir) = create_test_repo();
        let category = "Bug Fixes".to_string();
        let rule = Rule::new(category, r"(?i)\bbug\b", None).unwrap();
        let commit = create_test_commit(
            &repo,
            "master",
            "fix: a fix commit\n\nThis is the body of the commit containing bug",
        );
        let result = rule.eval(&commit);
        assert!(result);
    }

    #[test]
    fn test_rule_eval_title_no_match() {
        let (repo, _temp_dir) = create_test_repo();
        let category = "Bug Fixes".to_string();
        let rule = Rule::new(category, r"^fix: .+$", Some(CommitSection::Title)).unwrap();
        let commit = create_test_commit(&repo, "master", "chore: a chore commit");
        let result = rule.eval(&commit);
        assert!(!result);
    }

    #[test]
    fn test_rule_eval_body_no_match() {
        let (repo, _temp_dir) = create_test_repo();
        let category = "Breaking Changes".to_string();
        let rule = Rule::new(
            category,
            r"^BREAKING\sCHANGE:\s.+$",
            Some(CommitSection::Body),
        )
        .unwrap();
        let commit = create_test_commit(&repo, "master", "feat: a feature commit");
        let result = rule.eval(&commit);
        assert!(!result);
    }

    #[test]
    fn test_rule_eval_no_scope_no_match() {
        let (repo, _temp_dir) = create_test_repo();
        let category = "Bug Fixes".to_string();
        let rule = Rule::new(category, r"^fix: .+$", None).unwrap();
        let commit = create_test_commit(&repo, "master", "chore: a chore commit");
        let result = rule.eval(&commit);
        assert!(!result);
    }

    #[test]
    fn test_rule_eval_empty_commit_message() {
        let (repo, _temp_dir) = create_test_repo();
        let category = "Bug Fixes".to_string();
        let rule = Rule::new(category, r"^fix: .+$", None).unwrap();
        let commit = create_test_commit(&repo, "master", "");
        let result = rule.eval(&commit);
        assert!(!result);
    }
}
