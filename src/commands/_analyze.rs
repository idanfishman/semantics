// use std::path::Path;

// use anyhow::{Ok, Result};
// use git2::Repository;

// use crate::commit_analyzer::analyzer::Analyzer;
// use crate::config::Config;
// use crate::git::{collect_commits_from_head_to_tag, detect_current_branch};
// use crate::release_channel::{
//     ReleaseChannel, find_release_channel_by_branch, find_release_channel_by_name,
//     find_stable_release_channel,
// };
// use crate::semver_utils::{
//     INITIAL_STABLE_VERSION, bump_version, initial_prerelease_version, next_preprelease_version,
// };

// pub fn analyze(
//     config_path: &Path,
//     repo_path: &Path,
//     release_channel_name: Option<&str>,
// ) -> Result<()> {
//     let cfg = Config::from_file(config_path)?;
//     let repo = Repository::open(repo_path)?;

//     let analyzer = Analyzer::from_config(&cfg.commit_analyzer)?;

//     let target_channel: &ReleaseChannel = match release_channel_name {
//         Some(name) => find_release_channel_by_name(name, &cfg.release_channels)?,
//         None => {
//             let branch_name = detect_current_branch(&repo)?;
//             find_release_channel_by_branch(&branch_name, &cfg.release_channels)?
//         }
//     };

//     if target_channel.prerelease {
//         let stable_channel = find_stable_release_channel(&cfg.release_channels)?;
//         let stable_latest = stable_channel.latest_version(&repo, &cfg.tag_format)?;
//         let channel_latest = target_channel.latest_version(&repo, &cfg.tag_format)?;

//         match (stable_latest, channel_latest) {
//             (Some((stable_tag, stable_version)), Some((channel_tag, channel_version))) => {
//                 // Both stable and channel versions exist
//                 let commits = collect_commits_from_head_to_tag(&repo, &stable_tag)?;
//                 let bump = analyzer.analyze_commits(&commits);

//                 if bump.is_none() {
//                     println!("no changes detected, no version bump needed");
//                     return Ok(());
//                 }
//                 let bump = bump.unwrap();

//                 let new_version = next_preprelease_version(
//                     Some(&stable_version),
//                     Some(&channel_version),
//                     bump,
//                     &target_channel.name,
//                 );
//             }
//             (Some((stable_tag, stable_version)), None) => {
//                 // Only stable version exists
//                 let commits = collect_commits_from_head_to_tag(&repo, &stable_tag)?;
//                 let bump = analyzer.analyze_commits(&commits);

//                 if bump.is_none() {
//                     println!("no changes detected, no version bump needed");
//                     return Ok(());
//                 }
//                 let bump = bump.unwrap();

//                 let new_version = next_preprelease_version(
//                     Some(&stable_version),
//                     None,
//                     bump,
//                     &target_channel.name,
//                 );
//             }
//             (None, Some((channel_tag, channel_version))) => {
//                 // Only channel version exists (no stable)
//                 let commits = collect_commits_from_head_to_tag(&repo, &channel_tag)?;
//                 let bump = analyzer.analyze_commits(&commits);

//                 if bump.is_none() {
//                     println!("no changes detected, no version bump needed");
//                     return Ok(());
//                 }
//                 let bump = bump.unwrap();

//                 let new_version = next_preprelease_version(
//                     None,
//                     Some(&channel_version),
//                     bump,
//                     &target_channel.name,
//                 );
//             }
//             (None, None) => {
//                 let new_version = initial_prerelease_version(&target_channel.name);
//             }
//         }
//     } else {
//         let stable_latest = target_channel.latest_version(&repo, &cfg.tag_format)?;
//         match stable_latest {
//             Some((stable_tag, stable_version)) => {
//                 let commits = collect_commits_from_head_to_tag(&repo, &stable_tag)?;
//                 let bump = analyzer.analyze_commits(&commits);

//                 if bump.is_none() {
//                     println!("no changes detected, no version bump needed");
//                     return Ok(());
//                 }
//                 let bump = bump.unwrap();

//                 let new_version = bump_version(&stable_version, bump);
//             }
//             None => {
//                 let new_version = INITIAL_STABLE_VERSION.clone();
//             }
//         }
//     }

//     Ok(())
// }
