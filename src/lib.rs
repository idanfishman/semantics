use regex::Regex;
use semver::Version;

/// Captures and parses a semantic version from a tag string.
///
/// # Arguments
///
/// * `tag` - The tag string to parse (e.g., `"v1.2.3"` or `"1.2.3+build.1"`).
/// * `tag_prefix` - The prefix to trim from the tag (e.g., `"v"`).
/// * `pattern` - A compiled regex pattern to match semantic versions.
///
/// # Returns
///
/// Returns an `Option<semver::Version>` if the tag matches the pattern and can be parsed.
///
/// # Examples
///
/// ```
/// use regex::Regex;
/// use semver::Version;
/// use semantics::{capture_semver_version};
///
/// let SEMVER_REGEX = Regex::new(r"^(?:[^\d]*)(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([\w\.-]+))?(?:\+([\w\.-]+))?$").unwrap();
///
/// let version = capture_semver_version("v1.2.3", "v", &SEMVER_REGEX);
/// assert_eq!(version, Some(Version::new(1, 2, 3)));
///
/// let prerelease = capture_semver_version("v1.2.3-alpha", "v", &SEMVER_REGEX);
/// assert_eq!(prerelease, Some(Version::parse("1.2.3-alpha").unwrap()));
///
/// let build_metadata = capture_semver_version("v1.2.3+build.1", "v", &SEMVER_REGEX);
/// assert_eq!(build_metadata, Some(Version::parse("1.2.3+build.1").unwrap()));
///
/// let full_version = capture_semver_version("v1.2.3-alpha+build.1", "v", &SEMVER_REGEX);
/// assert_eq!(full_version, Some(Version::parse("1.2.3-alpha+build.1").unwrap()));
/// ```
pub fn capture_semver_version(tag: &str, tag_prefix: &str, pattern: &Regex) -> Option<Version> {
    pattern.captures(tag).and_then(|cap| {
        let version_str = cap.get(0)?.as_str();
        let trimmed_version = version_str.strip_prefix(tag_prefix).unwrap_or(version_str);
        trimmed_version.parse::<Version>().ok()
    })
}
