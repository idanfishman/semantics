use git2::Repository;
use release_channel::ReleaseChannel;
use semantics::capture_semver_version;

mod commit_analyzer;
mod config;
mod git;
mod release_channel;
mod utils;

#[cfg(test)]
mod test_helpers;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::Config::new();
    let repo = Repository::open(".").unwrap();

    let stable = release_channel::ReleaseChannel::new("stable", "main", false)?;
    let next: ReleaseChannel = release_channel::ReleaseChannel::new("next", "next", true)?;

    let branch_name = git::detect_current_branch(&repo)?;
    println!("Current branch: {}", branch_name);

    let tag_format = cfg.tag_format();
    let tag_prefix = &tag_format.replace("{version}", "");

    let semver_regex = regex::Regex::new(
        r"^(?:[^\d]*)(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([\w\.-]+))?(?:\+([\w\.-]+))?$",
    )
    .unwrap();

    let mut semver_tags: Vec<semver::Version> = repo
        .tag_names(None)
        .unwrap()
        .iter()
        .flatten()
        .filter_map(|tag| capture_semver_version(tag, tag_prefix, &semver_regex))
        .collect();

    semver_tags.sort_by(|a, b| b.cmp(a));

    println!("{:?}", semver_tags);
    println!("Latest version: {:?}", semver_tags.first());

    // collect the commits fro mthe latest tag to the HEAD to pass to the analyzer and changelog generator, than release using github release api

    let analyzer = commit_analyzer::analyzer::Analyzer::new(
        Some(commit_analyzer::analyzer::Preset::ConventionalCommits),
        None, // No additional rules provided
    )
    .unwrap();
    let analyzer_rules = analyzer.rules();
    println!("Analyzer rules: {:?}", analyzer_rules);

    Ok(())
}
