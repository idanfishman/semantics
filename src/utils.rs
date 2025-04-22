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

/// Constructs a pre-release identifier from the given head and tail.
/// The `head` is the pre-release label, and the `tail` is the pre-release number.
/// The pre-release identifier is formatted as `{head}.{tail}`.
///
/// # Arguments
///
/// * `head` - The pre-release label.
/// * `tail` - The pre-release number.
///
/// # Returns
///
/// * A `Prerelease` instance representing the pre-release identifier.
///
fn format_prerelease(head: &str, tail: u8) -> Prerelease {
    Prerelease::from_str(&format!("{}.{}", head, tail)).unwrap()
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
fn bump_version(version: &Version, bump: VersionBump) -> Version {
    match bump {
        VersionBump::Patch => Version::new(version.major, version.minor, version.patch + 1),
        VersionBump::Minor => Version::new(version.major, version.minor + 1, 0),
        VersionBump::Major => Version::new(version.major + 1, 0, 0),
    }
}

/// Determines the next stable version based on the current stable version and the specified bump type.
/// If the current stable version is `None`, it starts from `1.0.0`.
///
/// # Arguments
///
/// * `stable_version` - The current stable version.
/// * `bump` - The type of version bump to apply.
///
/// # Returns
///
/// * A new `Version` instance representing the next stable version.
pub fn next_stable_version(stable_version: Option<&Version>, bump: VersionBump) -> Version {
    match stable_version {
        Some(version) => bump_version(version, bump),
        None => Version::new(1, 0, 0),
    }
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
pub fn next_preprelease_version(
    stable_version: Option<&Version>,
    prerelease_version: Option<&Version>,
    bump: VersionBump,
    prerelease_label: &str,
) -> Version {
    match (stable_version, prerelease_version) {
        (None, None) => {
            // No stable or prerelease version, start from 1.0.0-{label}.1
            let mut new_version = next_stable_version(stable_version, bump);
            new_version.pre = format_prerelease(prerelease_label, 1);
            new_version
        }
        (None, Some(prerelease)) => {
            // No stable version, increment the prerelease version regardless of the bump type
            let mut prerelease_tail: u8 = prerelease
                .pre
                .as_str()
                .split('.')
                .last()
                .and_then(|s| s.parse::<u8>().ok()) // Safely parse the last element
                .unwrap();
            prerelease_tail += 1;

            let mut new_version = next_stable_version(stable_version, bump);
            new_version.pre = format_prerelease(prerelease_label, prerelease_tail);
            new_version
        }
        (Some(stable), None) => {
            // No prerelease version, start a new prerelease series
            let mut new_version = bump_version(stable, bump);
            new_version.pre = format_prerelease(prerelease_label, 1);
            new_version
        }
        (Some(stable), Some(prerelease)) => {
            if stable > prerelease {
                // Stable version is greater than prerelease, start a new prerelease series
                let mut new_version = bump_version(stable, bump);
                new_version.pre = format_prerelease(prerelease_label, 1);
                new_version
            } else {
                // Check if the bump is greater than the last prerelease version bump
                let mut new_version = bump_version(stable, bump);
                new_version.pre = format_prerelease(prerelease_label, 1);
                // Use the new bumped version if it is greater than the current prerelease version
                if new_version > *prerelease {
                    new_version
                } else {
                    // Increment the prerelease version
                    let mut prerelease_tail: u8 = prerelease
                        .pre
                        .as_str()
                        .split('.')
                        .last()
                        .and_then(|s| s.parse::<u8>().ok()) // Safely parse the last element
                        .unwrap();
                    prerelease_tail += 1;

                    let mut updated_version = prerelease.clone();
                    updated_version.pre = format_prerelease(prerelease_label, prerelease_tail);

                    updated_version
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{
        VersionBump, bump_version, format_prerelease, next_preprelease_version, next_stable_version,
    };
    use semver::Version;

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
    fn test_next_stable_version() {
        let version = Version::new(1, 2, 3);

        assert_eq!(
            next_stable_version(Some(&version), VersionBump::Patch),
            Version::new(1, 2, 4)
        );
        assert_eq!(
            next_stable_version(Some(&version), VersionBump::Minor),
            Version::new(1, 3, 0)
        );
        assert_eq!(
            next_stable_version(Some(&version), VersionBump::Major),
            Version::new(2, 0, 0)
        );
        assert_eq!(
            next_stable_version(None, VersionBump::Patch),
            Version::new(1, 0, 0)
        );
        assert_eq!(
            next_stable_version(None, VersionBump::Minor),
            Version::new(1, 0, 0)
        );
        assert_eq!(
            next_stable_version(None, VersionBump::Major),
            Version::new(1, 0, 0)
        );
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
        // This should panic because the prerelease label is invalid
        format_prerelease("invalid label!", 1);
    }

    #[test]
    fn test_next_preprelease_version() {
        let stable_version = Version::new(1, 0, 0);
        let prerelease_version = Version::parse("1.0.0-alpha.1").unwrap();

        // Case: No stable or prerelease version
        let version = next_preprelease_version(None, None, VersionBump::Patch, "alpha");
        assert_eq!(version.to_string(), "1.0.0-alpha.1");

        // Case: Only prerelease version
        let version =
            next_preprelease_version(None, Some(&prerelease_version), VersionBump::Patch, "alpha");
        assert_eq!(version.to_string(), "1.0.0-alpha.2");

        // Case: Only stable version
        let version =
            next_preprelease_version(Some(&stable_version), None, VersionBump::Patch, "alpha");
        assert_eq!(version.to_string(), "1.0.1-alpha.1");

        // Case: Stable > prerelease
        let version = next_preprelease_version(
            Some(&stable_version),
            Some(&prerelease_version),
            VersionBump::Patch,
            "alpha",
        );
        assert_eq!(version.to_string(), "1.0.1-alpha.1");

        // Case: Stable <= prerelease
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
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn test_next_preprelease_version_missing_tail() {
        // This should panic because the prerelease tail is missing or invalid
        let invalid_prerelease = Version::parse("1.0.0-alpha").unwrap();
        next_preprelease_version(None, Some(&invalid_prerelease), VersionBump::Patch, "alpha");
    }
}
