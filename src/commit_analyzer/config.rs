use serde::{Deserialize, Serialize};

use crate::commit_analyzer::preset::Preset;
use crate::commit_analyzer::rule::Rule;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
