use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct ReleaseChannel {
    name: String,
    branch: String,
    prerelease: bool,
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
    /// assert_eq!(channel.name(), "stable");
    /// assert_eq!(channel.branch(), "main");
    /// assert_eq!(channel.is_prerelease(), false);
    /// ```
    pub fn new(name: &str, branch: &str, prerelease: bool) -> Result<Self> {
        if name.is_empty() {
            anyhow::bail!("release channel name cannot be empty");
        }

        if branch.is_empty() {
            anyhow::bail!("release channel must be associated with a branch");
        }

        Ok(ReleaseChannel {
            name: name.to_string(),
            branch: branch.to_string(),
            prerelease,
        })
    }

    /// Returns a reference to the release channel name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the branch name.
    pub fn branch(&self) -> &str {
        &self.branch
    }

    /// Returns a boolean indicating if the release channel is a prerelease.
    pub fn prerelease(&self) -> bool {
        self.prerelease
    }
}

/// Finds the index of a release channel in a list of channels based on the branch name.
///
/// # Arguments
///
/// * `branch_name` - The name of the branch to search for.
/// * `channels` - A slice of `ReleaseChannel` instances.
///
/// # Returns
///
/// Returns a `Result` containing the index of the release channel if found, or an error message.
///
/// # Examples
///
/// ```rust
/// use crate::release_channel::{ReleaseChannel, find_release_channel};
///
/// let channels = vec![
///   ReleaseChannel::new("stable", "main", false).unwrap(),
///   ReleaseChannel::new("beta", "develop", true).unwrap(),
/// ];
///
/// let index = find_release_channel("main", &channels);
/// assert_eq!(index, Ok(0));
///
/// let index = find_release_channel("develop", &channels);
/// assert_eq!(index, Ok(1));
/// ```
///
/// # Errors
///
/// Returns an error if no release channel is found for the given branch name.
pub fn find_release_channel_by_branch(
    branch_name: &str,
    channels: &[ReleaseChannel],
) -> Result<usize> {
    channels
        .iter()
        .position(|channel| channel.branch() == branch_name)
        .with_context(|| format!("no release channel found with branch: {}", branch_name))
}

/// Finds the index of a release channel in a list of channels based on the channel name.
///
/// # Arguments
///
/// * `channel_name` - The name of the release channel to search for.
/// * `channels` - A slice of `ReleaseChannel` instances.
///
/// # Returns
///
/// Returns a `Result` containing the index of the release channel if found, or an error message.
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
/// let index = find_release_channel_by_name("stable", &channels).unwrap();
/// assert_eq!(index, 0);
///
/// let index = find_release_channel_by_name("beta", &channels).unwrap();
/// assert_eq!(index, 1);
/// ```
///
/// # Errors
///
/// Returns an error if no release channel is found for the given name.
pub fn find_release_channel_by_name(
    channel_name: &str,
    channels: &[ReleaseChannel],
) -> Result<usize> {
    channels
        .iter()
        .position(|channel| channel.name() == channel_name)
        .with_context(|| format!("no release channel found with name: {}", channel_name))
}

#[cfg(test)]
mod tests {
    use crate::release_channel::{
        ReleaseChannel, find_release_channel_by_branch, find_release_channel_by_name,
    };

    #[test]
    fn test_release_channel_new() {
        let channel = ReleaseChannel::new("stable", "main", false).unwrap();
        assert_eq!(channel.name(), "stable");
        assert_eq!(channel.branch(), "main");
        assert_eq!(channel.prerelease(), false);
    }

    #[test]
    fn test_release_channel_new_empty_name() {
        let result = ReleaseChannel::new("", "main", false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "release channel name cannot be empty"
        );
    }

    #[test]
    fn test_release_channel_new_empty_branch() {
        let result = ReleaseChannel::new("stable", "", false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "release channel must be associated with a branch"
        );
    }

    #[test]
    fn test_find_release_channel_by_branch() {
        let channels = vec![
            ReleaseChannel::new("stable", "main", false).unwrap(),
            ReleaseChannel::new("beta", "develop", true).unwrap(),
        ];

        let index = find_release_channel_by_branch("main", &channels).unwrap();
        assert_eq!(index, 0);

        let index = find_release_channel_by_branch("develop", &channels).unwrap();
        assert_eq!(index, 1);
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
            "no release channel found with branch: feature"
        );
    }

    #[test]
    fn test_find_release_channel_by_name() {
        let channels = vec![
            ReleaseChannel::new("stable", "main", false).unwrap(),
            ReleaseChannel::new("beta", "develop", true).unwrap(),
        ];

        let index = find_release_channel_by_name("stable", &channels).unwrap();
        assert_eq!(index, 0);

        let index = find_release_channel_by_name("beta", &channels).unwrap();
        assert_eq!(index, 1);
    }

    #[test]
    fn test_find_release_channel_by_name_not_found() {
        let channels = vec![
            ReleaseChannel::new("stable", "main", false).unwrap(),
            ReleaseChannel::new("beta", "develop", true).unwrap(),
        ];

        let index = find_release_channel_by_name("alpha", &channels);
        assert!(index.is_err());
        assert_eq!(
            index.unwrap_err().to_string(),
            "no release channel found with name: alpha"
        );
    }
}
