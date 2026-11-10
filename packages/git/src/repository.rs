use crate::error::Error;
use git2::{
    Direction, FetchOptions, Oid, Remote,
    build::{CheckoutBuilder, RepoBuilder},
};
use std::path::Path;

use crate::credentials::token_callbacks;

pub fn checkout(
    url: &str,
    path: &Path,
    commit_sha: &str,
    token: Option<&str>,
) -> Result<(), Error> {
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(token_callbacks(token));
    let repo = RepoBuilder::new().fetch_options(fetch_options).clone(url, path)?;
    let commit = repo.find_commit(Oid::from_str(commit_sha)?)?;
    repo.set_head_detached(commit.id())?;
    repo.checkout_head(Some(CheckoutBuilder::new().force()))?;
    Ok(())
}

pub fn get_last_commit(url: &str, branch: &str, token: Option<&str>) -> Result<String, Error> {
    resolve_revision(url, &format!("refs/heads/{branch}"), token)
}

/// Resolve HEAD, a branch, a tag, or a full commit SHA for a pinned checkout.
pub fn resolve_revision(url: &str, revision: &str, token: Option<&str>) -> Result<String, Error> {
    if revision.len() == 40
        && let Ok(commit) = Oid::from_str(revision)
    {
        return Ok(commit.to_string());
    }
    let mut remote = Remote::create_detached(url)?;
    let connection = remote.connect_auth(Direction::Fetch, Some(token_callbacks(token)), None)?;
    let references = if revision == "HEAD" || revision.starts_with("refs/") {
        vec![format!("{revision}^{{}}"), revision.to_owned()]
    } else {
        vec![
            format!("refs/heads/{revision}"),
            format!("refs/tags/{revision}^{{}}"),
            format!("refs/tags/{revision}"),
        ]
    };
    let heads = connection.list()?;
    for reference in references {
        if let Some(head) = heads.iter().find(|head| head.name() == reference) {
            return Ok(head.oid().to_string());
        }
    }
    Err(Error::Unexpected("Revision not found in remote repository"))
}
