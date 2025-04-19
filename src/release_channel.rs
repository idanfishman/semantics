use anyhow::{Context, Result};
use git2::Repository;
use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct ReleaseChannel {
    #[validate(length(min = 1, message = "release channel name cannot be empty"))]
    pub name: String,
    #[validate(length(min = 1, message = "release channel must be associated with a branch"))]
    pub branch: String,
    #[serde(default)]
    pub prerelease: bool,
}

impl ReleaseChannel {
    /// Creates a new instance of ReleaseChannel.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the release channel.
    /// * `branch` - The branch associated with the release channel.
    /// * `prerelease` - A boolean indicating if the release channel is a prerelease.
    ///
    /// # Errors
    ///
    /// Returns an error if the name is empty or if the branch is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::release_channel::ReleaseChannel;
    ///
    /// /// let channel = ReleaseChannel::new("stable", "main", false).unwrap();
    /// assert_eq!(channel.name, "stable");
    /// assert_eq!(channel.branch, "main");
    /// assert_eq!(channel.prerelease, false);
    /// ```
    pub fn new(name: &str, branch: &str, prerelease: bool) -> Result<Self> {
        let channel = ReleaseChannel {
            name: name.to_string(),
            branch: branch.to_string(),
            prerelease,
        };

        channel.validate()?;

        Ok(channel)
    }

    /// Gets the latest version from the Git repository based on the provided tag format.
    ///     
    /// # Arguments
    ///
    /// * `repo` - The Git repository to search for tags.
    /// * `tag_format` - The format of the tag (e.g., "v{version}").
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the latest version and its corresponding tag if successful,
    /// or an error message.
    ///
    pub fn latest_version(
        &self,
        repo: &Repository,
        tag_format: &str,
    ) -> Result<Option<(String, Version)>> {
        let versions = self.versions(repo, tag_format)?;
        Ok(versions.first().cloned())
    }

    /// Creates a regex pattern to match tags based on the provided tag format and release channel name.
    /// follows the original regex pattern from https://regex101.com/r/Ly7O1x/3/
    ///
    /// # Arguments
    ///
    /// * `tag_format` - The format of the tag (e.g., "v{version}").
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a regex pattern if successful, or an error message.
    ///
    fn tag_regex(&self, tag_prefix: &str) -> Result<Regex> {
        let base_regex = "(?P<major>0|[1-9]\\d*)\\.(?P<minor>0|[1-9]\\d*)\\.(?P<patch>0|[1-9]\\d*)";
        let prerelease_regex = match self.prerelease {
            true => format!(
                "(?:-(?P<prerelease>{}(?:\\.(?:0|[1-9]\\d*|\\d*[a-zA-Z-][0-9a-zA-Z-]*))*))",
                regex::escape(&self.name)
            ),
            false => String::new(),
        };
        let build_metadata_regex = "(?:\\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\\.[0-9a-zA-Z-]+)*))?";

        let tag_regex = format!(
            "^{}{}{}{}$",
            tag_prefix, base_regex, prerelease_regex, build_metadata_regex
        );

        Ok(Regex::new(&tag_regex)?)
    }

    /// Gets a list of versions from the Git repository based on the provided tag format.
    ///
    /// # Arguments
    ///
    /// * `repo` - The Git repository to search for tags.
    /// * `tag_format` - The format of the tag (e.g., "v{version}").
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of parsed versions and their corresponding tags if successful,
    ///
    fn versions(&self, repo: &Repository, tag_format: &str) -> Result<Vec<(String, Version)>> {
        let tag_prefix = tag_format.replace("{version}", "");
        let tag_regex = self.tag_regex(&tag_prefix)?;

        let mut semver_tags: Vec<(String, Version)> = repo
            .tag_names(None)?
            .iter()
            .flatten()
            .filter_map(|tag| {
                tag_regex.captures(tag).and_then(|cap| {
                    let version_str = cap.get(0)?.as_str();
                    let stripped_version =
                        version_str.strip_prefix(&tag_prefix).unwrap_or(version_str);
                    Version::parse(stripped_version)
                        .ok()
                        .map(|v| (tag.to_string(), v))
                })
            })
            .collect();

        // Sort by version in descending order
        semver_tags.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(semver_tags)
    }
}

/// Finds a release channel in a list of channels based on the branch name.
///
/// # Arguments
///
/// * `branch_name` - The name of the branch to search for.
/// * `channels` - A slice of `ReleaseChannel` instances.
///
/// # Returns
///
/// Returns a `Result` containing a reference to the release channel if found, or an error message.
///
/// # Examples
///
/// ```rust
/// use crate::release_channel::{ReleaseChannel, find_release_channel_by_branch};
///
/// let channels = vec![
///   ReleaseChannel::new("stable", "main", false).unwrap(),
///   ReleaseChannel::new("beta", "develop", true).unwrap(),
/// ];
///
/// let channel = find_release_channel_by_branch("main", &channels).unwrap();
/// assert_eq!(channel.name, "stable");
///
/// let channel = find_release_channel_by_branch("develop", &channels).unwrap();
/// assert_eq!(channel.name, "beta");
/// ```
///
pub fn find_release_channel_by_branch<'a>(
    branch_name: &str,
    channels: &'a [ReleaseChannel],
) -> Result<&'a ReleaseChannel> {
    channels
        .iter()
        .find(|channel| channel.branch == branch_name)
        .with_context(|| format!("no release channel found for branch: {}", branch_name))
}

/// Finds a release channel in a list of channels based on the channel name.
///
/// # Arguments
///
/// * `channel_name` - The name of the release channel to search for.
/// * `channels` - A slice of `ReleaseChannel` instances.
///
/// # Returns
///
/// Returns a `Result` containing a reference to the release channel if found, or an error message.
///
/// # Examples
///
/// ```rust
/// use crate::release_channel::{ReleaseChannel, find_release_channel_by_name};
///
/// let channels = vec![
///   ReleaseChannel::new("stable", "main", false).unwrap(),
///   ReleaseChannel::new("beta", "develop", true).unwrap(),
/// ];
///
/// let channel = find_release_channel_by_name("stable", &channels).unwrap();
/// assert_eq!(channel.name, "stable");
///
/// let channel = find_release_channel_by_name("beta", &channels).unwrap();
/// assert_eq!(channel.name, "beta");
/// ```
///
pub fn find_release_channel_by_name<'a>(
    channel_name: &str,
    channels: &'a [ReleaseChannel],
) -> Result<&'a ReleaseChannel> {
    channels
        .iter()
        .find(|channel| channel.name == channel_name)
        .with_context(|| format!("no release channel found with name: {}", channel_name))
}

/// Finds the stable release channel in a list of channels.
///
/// # Arguments
///
/// * `channels` - A slice of `ReleaseChannel` instances.
///
/// # Returns
///
/// Returns a `Result` containing a reference to the stable release channel if found, or an error message.
///
/// # Examples
///
/// ```rust
/// use crate::release_channel::{ReleaseChannel, find_stable_release_channel};
///     
/// let channels = vec![
///   ReleaseChannel::new("stable", "main", false).unwrap(),
///   ReleaseChannel::new("beta", "develop", true).unwrap(),
/// ];
///
/// let channel = find_stable_release_channel(&channels).unwrap();
/// assert_eq!(channel.name, "stable");
/// assert_eq!(channel.branch, "main");
/// assert_eq!(channel.prerelease, false);
/// /// ```
///
///
pub fn find_stable_release_channel<'a>(
    channels: &'a [ReleaseChannel],
) -> Result<&'a ReleaseChannel> {
    channels
        .iter()
        .find(|channel| !channel.prerelease)
        .with_context(|| "no stable release channel found")
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use crate::release_channel::{
        ReleaseChannel, find_release_channel_by_branch, find_release_channel_by_name,
        find_stable_release_channel,
    };
    use crate::test_helpers::create_versioned_test_repo;

    #[test]
    fn test_release_channel_new() {
        let channel = ReleaseChannel::new("stable", "main", false).unwrap();
        assert_eq!(channel.name, "stable");
        assert_eq!(channel.branch, "main");
        assert_eq!(channel.prerelease, false);
    }

    #[test]
    fn test_release_channel_new_empty_name() {
        let result = ReleaseChannel::new("", "main", false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "name: release channel name cannot be empty"
        );
    }

    #[test]
    fn test_release_channel_new_empty_branch() {
        let result = ReleaseChannel::new("stable", "", false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "branch: release channel must be associated with a branch"
        );
    }

    #[test]
    fn test_tag_regex_prerelease_false() {
        let channel = ReleaseChannel::new("stable", "main", false).unwrap();
        let regex = channel.tag_regex("v").unwrap();
        // Should match v1.2.3, but not v1.2.3-alpha
        assert!(regex.is_match("v1.2.3"));
        assert!(!regex.is_match("v1.2.3-alpha"));
        assert!(!regex.is_match("v1.2.3-stable"));
    }

    #[test]
    fn test_tag_regex_prerelease_true() {
        let channel = ReleaseChannel::new("alpha", "main", true).unwrap();
        let regex = channel.tag_regex("v").unwrap();
        // Should match v1.2.3-alpha, v1.2.3-alpha.1, v1.2.3-alpha.1.2
        assert!(regex.is_match("v1.2.3-alpha"));
        assert!(regex.is_match("v1.2.3-alpha.1"));
        assert!(regex.is_match("v1.2.3-alpha.1.2"));
        // Should not match v1.2.3-beta.1 or v1.2.3
        assert!(!regex.is_match("v1.2.3-beta.1"));
        assert!(!regex.is_match("v1.2.3"));
    }

    #[test]
    fn test_versions_and_latest_version() {
        let tags_and_msgs = vec![
            ("v1.0.0", "first release"),
            ("v1.1.0", "minor release"),
            ("v2.0.0", "major release"),
        ];
        let (repo, _dir) = create_versioned_test_repo(&tags_and_msgs);
        let channel = ReleaseChannel::new("stable", "master", false).unwrap();
        let versions = channel.versions(&repo, "v{version}").unwrap();
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0].1, Version::parse("2.0.0").unwrap());
        assert_eq!(versions[1].1, Version::parse("1.1.0").unwrap());
        assert_eq!(versions[2].1, Version::parse("1.0.0").unwrap());

        let latest = channel.latest_version(&repo, "v{version}").unwrap();
        assert_eq!(latest.unwrap().1, Version::parse("2.0.0").unwrap());
    }

    #[test]
    fn test_versions_and_latest_version_prerelease() {
        let tags_and_msgs = vec![
            ("v1.0.0-alpha.1", "alpha prerelease"),
            ("v1.0.0-alpha.2", "alpha prerelease 2"),
            ("v1.0.0-beta.1", "beta prerelease"),
        ];
        let (repo, _dir) = create_versioned_test_repo(&tags_and_msgs);
        let alpha_channel = ReleaseChannel::new("alpha", "master", true).unwrap();
        let beta_channel = ReleaseChannel::new("beta", "master", true).unwrap();
        let alpha_versions = alpha_channel.versions(&repo, "v{version}").unwrap();
        assert_eq!(alpha_versions.len(), 2);
        assert_eq!(
            alpha_versions[0].1,
            Version::parse("1.0.0-alpha.2").unwrap()
        );
        assert_eq!(
            alpha_versions[1].1,
            Version::parse("1.0.0-alpha.1").unwrap()
        );
        let beta_versions = beta_channel.versions(&repo, "v{version}").unwrap();
        assert_eq!(beta_versions.len(), 1);
        assert_eq!(beta_versions[0].1, Version::parse("1.0.0-beta.1").unwrap());
        let latest = alpha_channel.latest_version(&repo, "v{version}").unwrap();
        assert_eq!(latest.unwrap().1, Version::parse("1.0.0-alpha.2").unwrap());
    }

    #[test]
    fn test_find_release_channel_by_branch() {
        let channels = vec![
            ReleaseChannel::new("stable", "main", false).unwrap(),
            ReleaseChannel::new("beta", "develop", true).unwrap(),
        ];

        let channel = find_release_channel_by_branch("main", &channels).unwrap();
        assert_eq!(channel.name, "stable");
        assert_eq!(channel.branch, "main");
        assert_eq!(channel.prerelease, false);

        let channel = find_release_channel_by_branch("develop", &channels).unwrap();
        assert_eq!(channel.name, "beta");
        assert_eq!(channel.branch, "develop");
        assert_eq!(channel.prerelease, true);
    }

    #[test]
    fn test_find_release_channel_by_branch_not_found() {
        let channels = vec![
            ReleaseChannel::new("stable", "main", false).unwrap(),
            ReleaseChannel::new("beta", "develop", true).unwrap(),
        ];

        let index = find_release_channel_by_branch("feature", &channels);
        assert!(index.is_err());
        assert_eq!(
            index.unwrap_err().to_string(),
            "no release channel found for branch: feature"
        );
    }

    #[test]
    fn test_find_release_channel_by_branch_multiple() {
        let channels = vec![
            ReleaseChannel::new("stable", "main", false).unwrap(),
            ReleaseChannel::new("rc", "main", true).unwrap(),
        ];

        let channel = find_release_channel_by_branch("main", &channels).unwrap();
        assert_eq!(channel.name, "stable"); // Verifies the first match
        assert_eq!(channel.branch, "main");
        assert_eq!(channel.prerelease, false);
    }

    #[test]
    fn test_find_release_channel_by_name() {
        let channels = vec![
            ReleaseChannel::new("stable", "main", false).unwrap(),
            ReleaseChannel::new("beta", "develop", true).unwrap(),
        ];

        let channel = find_release_channel_by_name("stable", &channels).unwrap();
        assert_eq!(channel.name, "stable");
        assert_eq!(channel.branch, "main");
        assert_eq!(channel.prerelease, false);

        let channel = find_release_channel_by_name("beta", &channels).unwrap();
        assert_eq!(channel.name, "beta");
        assert_eq!(channel.branch, "develop");
        assert_eq!(channel.prerelease, true);
    }

    #[test]
    fn test_find_release_channel_by_name_not_found() {
        let channels = vec![
            ReleaseChannel::new("stable", "main", false).unwrap(),
            ReleaseChannel::new("beta", "develop", true).unwrap(),
        ];

        let result = find_release_channel_by_name("alpha", &channels);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "no release channel found with name: alpha"
        );
    }

    #[test]
    fn test_find_release_channel_by_name_multiple() {
        let channels = vec![
            ReleaseChannel::new("stable", "main", false).unwrap(),
            ReleaseChannel::new("stable", "develop", true).unwrap(),
        ];

        let channel = find_release_channel_by_name("stable", &channels).unwrap();
        assert_eq!(channel.name, "stable");
        assert_eq!(channel.branch, "main"); // Verifies the first match
        assert_eq!(channel.prerelease, false);
    }

    #[test]
    fn test_find_stable_release_channel() {
        let channels = vec![
            ReleaseChannel::new("stable", "main", false).unwrap(),
            ReleaseChannel::new("beta", "develop", true).unwrap(),
        ];

        let channel = find_stable_release_channel(&channels).unwrap();
        assert_eq!(channel.name, "stable");
        assert_eq!(channel.branch, "main");
        assert_eq!(channel.prerelease, false);
    }

    #[test]
    fn test_find_stable_release_channel_not_found() {
        let channels = vec![
            ReleaseChannel::new("beta", "develop", true).unwrap(),
            ReleaseChannel::new("alpha", "feature", true).unwrap(),
        ];

        let result = find_stable_release_channel(&channels);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "no stable release channel found"
        );
    }
}
