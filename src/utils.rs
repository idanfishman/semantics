use once_cell::sync::Lazy;
use semver::{Prerelease, Version};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

/// The initial stable version of the application.
///
/// This is lazily initialized to `1.0.0` and can be used as a base version
/// for creating prerelease or stable versions.
pub static INITIAL_STABLE_VERSION: Lazy<Version> = Lazy::new(|| Version::new(1, 0, 0));

/// Creates the initial prerelease version based on the given label.
///
/// # Arguments
///
/// * `label` - The prerelease label (e.g., "alpha", "beta").
///
/// # Returns
///
/// * A `Version` instance representing the initial prerelease version (e.g., `1.0.0-alpha.1`).
pub fn initial_prerelease_version(label: &str) -> Version {
    create_prerelease_version(&INITIAL_STABLE_VERSION, label, 1)
}

/// Returns the next version based on the current version and the specified bump type.
/// The `bump` parameter determines how the version should be incremented.
///
/// # Arguments
///
/// * `version` - The current version.
/// * `bump` - The type of version bump to apply.
///
/// # Returns
///
/// * A new `Version` instance with the incremented version number.
///
pub fn bump_version(version: &Version, bump: VersionBump) -> Version {
    match bump {
        VersionBump::Patch => Version::new(version.major, version.minor, version.patch + 1),
        VersionBump::Minor => Version::new(version.major, version.minor + 1, 0),
        VersionBump::Major => Version::new(version.major + 1, 0, 0),
    }
}

/// Constructs a prerelease version from the given base version, label, and tail.
///
/// # Arguments
///
/// * `base_version` - The base version.
/// * `label` - The prerelease label.
/// * `tail` - The prerelease number.
///
/// # Returns
///
/// * A new `Version` instance with the prerelease identifier.
///
fn create_prerelease_version(base_version: &Version, label: &str, tail: u8) -> Version {
    let mut new_version = base_version.clone();
    new_version.pre = format_prerelease(label, tail);
    new_version
}

/// Constructs a prerelease identifier from the given head and tail.
///
/// # Arguments
///
/// * `head` - The prerelease label.
/// * `tail` - The prerelease number.
///
/// # Returns
///
/// * A `Prerelease` instance representing the prerelease identifier.
///
fn format_prerelease(head: &str, tail: u8) -> Prerelease {
    Prerelease::from_str(&format!("{}.{}", head, tail)).unwrap()
}

/// Increments the prerelease tail of the given version.
///
/// # Arguments
///
/// * `version` - The current version.
///
/// # Returns
///
/// * The incremented prerelease tail as a `u8`.
///
/// # Panics
///
/// * Panics if the prerelease format is not numeric.
///
fn inc_prerelease_tail(version: &Version) -> u8 {
    version
        .pre
        .as_str()
        .split('.')
        .last()
        .and_then(|s| s.parse::<u8>().ok()) // Safely parse the last element
        .unwrap_or_else(|| {
            panic!(
                "unsupported prerelease format: tails must be numeric. found: {}",
                version.pre
            )
        })
        + 1
}

/// Determines the next pre-release version based on the current stable and pre-release versions.
///
/// # Arguments
///
/// * `stable_version` - The current stable version
/// * `prerelease_version` - The current pre-release version
/// * `bump` - The type of version bump to apply
/// * `prerelease_label` - The label for the pre-release version
///
/// # Returns
///
/// * A new `Version` instance representing the next pre-release version.
///
/// # Panics
///
/// * Panics if both `stable_version` and `prerelease_version` are `None`.
/// * Panics if the prerelease tail is not numeric.
///
pub fn next_preprelease_version(
    stable_version: Option<&Version>,
    prerelease_version: Option<&Version>,
    bump: VersionBump,
    prerelease_label: &str,
) -> Version {
    match (stable_version, prerelease_version) {
        (None, None) => {
            panic!(
                "both stable and prerelease versions are None. at least one version must be provided."
            );
        }
        (None, Some(prerelease)) => {
            // No stable version, increment the prerelease version regardless of the bump type
            create_prerelease_version(
                prerelease,
                prerelease_label,
                inc_prerelease_tail(prerelease),
            )
        }
        (Some(stable), None) => {
            // No prerelease version, start a new prerelease series
            create_prerelease_version(&bump_version(stable, bump), prerelease_label, 1)
        }
        (Some(stable), Some(prerelease)) => {
            if stable > prerelease {
                // Stable version is greater than prerelease, start a new prerelease series
                create_prerelease_version(&bump_version(stable, bump), prerelease_label, 1)
            } else {
                let bumped_version =
                    create_prerelease_version(&bump_version(stable, bump), prerelease_label, 1);

                // If the bumped version is greater than the current prerelease version use it
                if bumped_version > *prerelease {
                    bumped_version
                } else {
                    // Otherwise increment the prerelease version
                    create_prerelease_version(
                        prerelease,
                        prerelease_label,
                        inc_prerelease_tail(prerelease),
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use crate::utils::{
        VersionBump, bump_version, create_prerelease_version, format_prerelease,
        inc_prerelease_tail, initial_prerelease_version, next_preprelease_version,
    };

    #[test]
    fn test_initial_prerelease_version() {
        let prerelease = initial_prerelease_version("alpha");
        assert_eq!(prerelease.to_string(), "1.0.0-alpha.1");

        let prerelease = initial_prerelease_version("beta");
        assert_eq!(prerelease.to_string(), "1.0.0-beta.1");
    }

    #[test]
    fn test_bump_version() {
        let version = Version::new(1, 2, 3);

        assert_eq!(
            bump_version(&version, VersionBump::Patch),
            Version::new(1, 2, 4)
        );
        assert_eq!(
            bump_version(&version, VersionBump::Minor),
            Version::new(1, 3, 0)
        );
        assert_eq!(
            bump_version(&version, VersionBump::Major),
            Version::new(2, 0, 0)
        );
    }

    #[test]
    fn test_create_prerelease_version() {
        let base_version = Version::new(1, 0, 0);
        let prerelease = create_prerelease_version(&base_version, "alpha", 1);
        assert_eq!(prerelease.to_string(), "1.0.0-alpha.1");

        let prerelease = create_prerelease_version(&base_version, "beta", 42);
        assert_eq!(prerelease.to_string(), "1.0.0-beta.42");

        let prerelease_alpha = create_prerelease_version(&base_version, "alpha", 1);
        assert_eq!(prerelease_alpha.to_string(), "1.0.0-alpha.1");
    }

    #[test]
    fn test_format_prerelease() {
        let prerelease = format_prerelease("alpha", 1);
        assert_eq!(prerelease.as_str(), "alpha.1");

        let prerelease = format_prerelease("beta", 42);
        assert_eq!(prerelease.as_str(), "beta.42");
    }

    #[test]
    #[should_panic(expected = "unexpected character in pre-release identifier")]
    fn test_format_prerelease_invalid() {
        format_prerelease("invalid label!", 1);
    }

    #[test]
    fn test_inc_prerelease_tail() {
        let version = Version::parse("1.0.0-alpha.1").unwrap();
        assert_eq!(inc_prerelease_tail(&version), 2);

        let version = Version::parse("1.0.0-beta.42").unwrap();
        assert_eq!(inc_prerelease_tail(&version), 43);
    }

    #[test]
    #[should_panic(
        expected = "both stable and prerelease versions are None. at least one version must be provided."
    )]
    fn test_next_preprelease_version_missing_versions() {
        next_preprelease_version(None, None, VersionBump::Patch, "alpha");
    }

    #[test]
    fn test_next_preprelease_version_only_prerelease() {
        let prerelease_version = Version::parse("1.0.0-alpha.1").unwrap();
        let version =
            next_preprelease_version(None, Some(&prerelease_version), VersionBump::Patch, "alpha");
        assert_eq!(version.to_string(), "1.0.0-alpha.2");
    }

    #[test]
    fn test_next_preprelease_version_only_stable() {
        let stable_version = Version::new(1, 0, 0);
        let version =
            next_preprelease_version(Some(&stable_version), None, VersionBump::Patch, "alpha");
        assert_eq!(version.to_string(), "1.0.1-alpha.1");
    }

    #[test]
    fn test_next_preprelease_version_stable_greater_than_prerelease() {
        let stable_version = Version::new(1, 0, 0);
        let prerelease_version = Version::parse("1.0.0-alpha.1").unwrap();
        let version = next_preprelease_version(
            Some(&stable_version),
            Some(&prerelease_version),
            VersionBump::Patch,
            "alpha",
        );
        assert_eq!(version.to_string(), "1.0.1-alpha.1");
    }

    #[test]
    fn test_next_preprelease_version_stable_equal_to_prerelease() {
        let stable_version = Version::new(1, 0, 0);
        let prerelease_version = Version::parse("1.0.1-alpha.1").unwrap();
        let version = next_preprelease_version(
            Some(&stable_version),
            Some(&prerelease_version),
            VersionBump::Patch,
            "alpha",
        );
        assert_eq!(version.to_string(), "1.0.1-alpha.2");
    }

    #[test]
    fn test_next_preprelease_version_stable_less_than_prerelease() {
        let stable_version = Version::new(1, 0, 0);
        let bumped_prerelease = Version::parse("1.1.0-alpha.2").unwrap();
        let version = next_preprelease_version(
            Some(&stable_version),
            Some(&bumped_prerelease),
            VersionBump::Patch,
            "alpha",
        );
        assert_eq!(version.to_string(), "1.1.0-alpha.3");
    }

    #[test]
    #[should_panic(expected = "unsupported prerelease format: tails must be numeric. found: alpha")]
    fn test_next_preprelease_version_missing_tail() {
        let invalid_prerelease = Version::parse("1.0.0-alpha").unwrap();
        next_preprelease_version(None, Some(&invalid_prerelease), VersionBump::Patch, "alpha");
    }
}
