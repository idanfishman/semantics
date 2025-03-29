#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VersionBump {
    /// Increment the patch version (backwards compatible fixes).
    Patch,
    /// Increment the minor version (new features, backwards compatible).
    Minor,
    /// Increment the major version (breaking changes).
    Major,
}
