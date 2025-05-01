use anyhow::{Context, Ok, Result};
use git2::{Commit, Oid, Repository};

/// Collects commits from the HEAD to a specified tag in a Git repository.
/// Sorts the commits in reverse chronological order. (from tag to HEAD)
///     
/// # Arguments
///
/// * `repo` - A reference to a `git2::Repository` object.
/// * `tag` - tag name.
///
/// # Returns
///
/// vector of commits from HEAD to the specified tag.
pub fn collect_commits_from_head_to_tag<'repo>(
    repo: &'repo Repository,
    tag: &str,
) -> Result<Vec<Commit<'repo>>> {
    let tag_oid = resolve_tag_oid(repo, tag)?;
    let head_oid = resolve_head_oid(repo)?;

    if !repo.graph_descendant_of(head_oid, tag_oid)? {
        anyhow::bail!("tag '{}' is not an ancestor of HEAD", tag);
    }

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)?;
    revwalk.push(head_oid)?;
    revwalk.hide(tag_oid)?;

    let commits: Vec<Commit<'_>> = revwalk
        .map(|oid| {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            Ok(commit)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(commits)
}

/// Resolves the OID of a tag.
fn resolve_tag_oid(repo: &Repository, tag: &str) -> Result<Oid> {
    let tag_ref = format!("refs/tags/{}", tag);
    repo.revparse_single(&tag_ref)
        .map(|obj| obj.id())
        .context(format!("could not find tag {}", tag))
}

/// Resolves the OID of the HEAD commit.
fn resolve_head_oid(repo: &Repository) -> Result<Oid> {
    if repo.head_detached()? {
        anyhow::bail!("detached HEAD is not supported");
    }

    repo.head()?
        .target()
        .ok_or_else(|| anyhow::anyhow!("HEAD is not pointing to a commit"))
}

/// Detects the current branch of a Git repository.
///
/// # Arguments
///
/// * `repo` - A reference to a `git2::Repository` object.
///
/// # Returns
///
/// The name of the current branch.
///
/// # Errors
///
/// This function returns an error in the following cases:
/// - The repository is in a detached HEAD state.
/// - The branch name cannot be determined (e.g., the branch is unnamed).
/// - The HEAD reference cannot be retrieved (e.g., the repository is in an invalid state).
///
/// # Examples
///
/// ```rust
/// use git2::Repository;
/// use crate::git::detect_current_branch;
///
/// // Open a Git repository
/// let repo = Repository::open(".").unwrap();
///
/// // Detect the current branch
/// match detect_current_branch(&repo) {
///     Ok(branch_name) => println!("Current branch: {}", branch_name),
///     Err(err) => eprintln!("Error: {}", err),
/// }
/// ```
pub fn detect_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head().context("could not get HEAD reference")?;

    if head.is_branch() {
        head.shorthand()
            .map(ToString::to_string)
            .context("could not determine branch name")
    } else {
        anyhow::bail!("detached HEAD state, not on any branch");
    }
}

#[cfg(test)]
mod tests {
    use crate::git::{collect_commits_from_head_to_tag, detect_current_branch};
    use crate::test_helpers::{create_test_commit, create_test_repo, create_test_tag};

    #[test]
    fn test_collect_commits_from_head_to_tag_multiple_commits() {
        let (repo, _temp_dir) = create_test_repo();
        let commit = create_test_commit(&repo, "master", "Initial commit");
        let tag = "v1.0.0";
        create_test_tag(&repo, &commit, tag);
        create_test_commit(&repo, "master", "Second commit");
        create_test_commit(&repo, "master", "Third commit");
        let commits = collect_commits_from_head_to_tag(&repo, tag).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].message().unwrap(), "Second commit");
        assert_eq!(commits[1].message().unwrap(), "Third commit");
    }

    #[test]
    fn test_collect_commits_from_head_to_tag_not_ancestor() {
        let (repo, _temp_dir) = create_test_repo();
        let commit1 = create_test_commit(&repo, "master", "Initial commit");
        let tag = "v1.0.0";
        create_test_tag(&repo, &commit1, tag);
        // Reset HEAD to a commit that is not an ancestor of the tag
        repo.reset(
            &repo.find_object(commit1.id(), None).unwrap(),
            git2::ResetType::Hard,
            None,
        )
        .unwrap();
        let result = collect_commits_from_head_to_tag(&repo, tag);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!("tag '{}' is not an ancestor of HEAD", tag)
        );
    }

    #[test]
    fn test_collect_commits_from_head_to_tag_no_commits() {
        let (repo, _temp_dir) = create_test_repo();
        let commit = create_test_commit(&repo, "master", "Initial commit");
        let tag = "v1.0.0";
        create_test_tag(&repo, &commit, tag);
        let result = collect_commits_from_head_to_tag(&repo, tag);
        // There should be no commits between HEAD and the tag
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!("tag '{}' is not an ancestor of HEAD", tag)
        );
    }

    #[test]
    fn test_collect_commits_from_head_to_tag_not_found() {
        let (repo, _temp_dir) = create_test_repo();
        // Attempt to collect commits from a non-existent tag
        let tag = "v1.0.0";
        let result = collect_commits_from_head_to_tag(&repo, tag);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains(&format!("could not find tag {}", tag))
        );
    }

    #[test]
    fn test_collect_commits_from_head_to_tag_detached_head() {
        let (repo, _temp_dir) = create_test_repo();
        let commit1 = create_test_commit(&repo, "master", "Initial commit");
        let tag = "v1.0.0";
        create_test_tag(&repo, &commit1, tag);
        let commit2 = create_test_commit(&repo, "master", "Second commit");
        repo.set_head_detached(commit2.id()).unwrap();
        repo.checkout_head(None).unwrap();
        let result = collect_commits_from_head_to_tag(&repo, tag);
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result.unwrap_err().to_string(),
            "detached HEAD is not supported"
        );
    }

    #[test]
    fn test_git_detect_current_branch() {
        let (repo, _temp_dir) = create_test_repo();
        let branch_name = detect_current_branch(&repo).unwrap();
        assert_eq!(branch_name, "master");
    }

    #[test]
    fn test_git_detect_current_branch_detached() {
        let (repo, _temp_dir) = create_test_repo();
        let commit = create_test_commit(&repo, "master", "detached commit");
        // Detach HEAD by setting it to the commit directly
        repo.set_head_detached(commit.id()).unwrap();
        let result = detect_current_branch(&repo);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "detached HEAD state, not on any branch"
        );
    }

    #[test]
    fn test_git_detect_current_branch_no_head() {
        let (repo, _temp_dir) = create_test_repo();
        // Simulate a repository without a HEAD reference
        repo.set_head("refs/heads/nonexistent").unwrap();
        let result = detect_current_branch(&repo);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("could not get HEAD reference")
        );
    }
}
