#[cfg(test)]
use git2::{Commit, Repository};
use tempfile::{TempDir, tempdir};

#[cfg(test)]
pub fn create_test_repo() -> (Repository, TempDir) {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();

    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "idanfishman").unwrap();
        config.set_str("user.email", "idan@fishman.dev").unwrap();
    }

    {
        let sig = repo.signature().unwrap();
        let tree_oid = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        if repo.head().is_err() {
            repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();
        }
    }

    (repo, dir)
}

#[cfg(test)]
pub fn create_test_commit<'repo>(
    repo: &'repo Repository,
    branch: &str,
    message: &str,
) -> Commit<'repo> {
    let tree_oid = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = repo.signature().unwrap();

    let branch_ref = format!("refs/heads/{}", branch);

    let reference = repo.find_reference(&branch_ref).unwrap();
    let parent_commit = reference.peel_to_commit().unwrap();

    let commit_oid = repo
        .commit(
            Some(&branch_ref),
            &sig,
            &sig,
            message,
            &tree,
            &[&parent_commit],
        )
        .unwrap();

    repo.find_commit(commit_oid).unwrap()
}

#[cfg(test)]
pub fn create_test_tag<'repo>(repo: &'repo Repository, commit: &'repo Commit, tag_name: &str) {
    repo.tag_lightweight(tag_name, commit.as_object(), true)
        .unwrap();
}
