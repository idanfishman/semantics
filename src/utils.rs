use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum VersionBump {
    /// Increment the patch version (backwards compatible fixes).
    #[serde(rename = "patch")]
    Patch,
    /// Increment the minor version (new features, backwards compatible).
    #[serde(rename = "minor")]
    Minor,
    /// Increment the major version (breaking changes).
    #[serde(rename = "major")]
    Major,
}
