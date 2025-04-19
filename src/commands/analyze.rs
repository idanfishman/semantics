use std::path::Path;

use anyhow::{Ok, Result};
use git2::Repository;

use crate::commit_analyzer::analyzer::Analyzer;
use crate::config::Config;
use crate::git::{collect_commits_from_head_to_tag, detect_current_branch};
use crate::release_channel::{
    ReleaseChannel, find_release_channel_by_branch, find_release_channel_by_name,
    find_stable_release_channel,
};

pub fn analyze(
    config_path: &Path,
    repo_path: &Path,
    release_channel_name: Option<&str>,
) -> Result<()> {
    let cfg = Config::from_file(config_path)?;
    let repo = Repository::open(repo_path)?;

    let analyzer = Analyzer::from_config(&cfg.commit_analyzer)?;

    let target_channel: &ReleaseChannel = match release_channel_name {
        Some(name) => find_release_channel_by_name(name, &cfg.release_channels)?,
        None => {
            let branch_name = detect_current_branch(&repo)?;
            find_release_channel_by_branch(&branch_name, &cfg.release_channels)?
        }
    };

    if target_channel.prerelease {
        let stable_channel = find_stable_release_channel(&cfg.release_channels)?;
        let stable_latest = stable_channel.latest_version(&repo, &cfg.tag_format)?;
        let channel_latest = target_channel.latest_version(&repo, &cfg.tag_format)?;

        match (stable_latest, channel_latest) {
            (Some((stable_tag, stable_ver)), Some((channel_tag, channel_ver))) => {
                // Both stable and channel versions exist
                let commits = collect_commits_from_head_to_tag(&repo, &stable_tag)?;
                let bump = analyzer.analyze_commits(&commits);
                // TODO: Decide: increment prerelease or start new prerelease series
                // You have stable_tag, stable_ver, channel_tag, channel_ver, bump
            }
            (Some((stable_tag, stable_ver)), None) => {
                // Only stable version exists
                let commits = collect_commits_from_head_to_tag(&repo, &stable_tag)?;
                let bump = analyzer.analyze_commits(&commits);
                // TODO: Bump base version according to changes, start prerelease series
            }
            (None, Some((channel_tag, channel_ver))) => {
                // Only channel version exists (no stable)
                let commits = collect_commits_from_head_to_tag(&repo, &channel_tag)?;
                let bump = analyzer.analyze_commits(&commits);
                // TODO: If a bigger bump is needed, start new prerelease series
            }
            (None, None) => {
                // Neither stable nor channel version exists
                // No commits to collect, just start at 1.0.0
                // TODO: Set version to 1.0.0 and create a version tag using the tag format and release channel name
            }
        }
    } else {
        let stable_latest = target_channel.latest_version(&repo, &cfg.tag_format)?;
        match stable_latest {
            Some((stable_tag, stable_ver)) => {
                let commits = collect_commits_from_head_to_tag(&repo, &stable_tag)?;
                let bump = analyzer.analyze_commits(&commits);
                // TODO: Create a version tag using the tag format
            }
            None => {
                // No commits to collect, just start at 1.0.0
                // TODO: Set version to 1.0.0 and create a version tag using the tag format
            }
        }
    }

    Ok(())
}
