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
    let mut remote = Remote::create_detached(url)?;
    let connection = remote.connect_auth(Direction::Fetch, Some(token_callbacks(token)), None)?;
    let reference = format!("refs/heads/{branch}");
    let head = connection
        .list()?
        .iter()
        .find(|head| head.name() == reference)
        .ok_or(Error::Unexpected("Branch not found in remote repository"))?;
    Ok(head.oid().to_string())
}
