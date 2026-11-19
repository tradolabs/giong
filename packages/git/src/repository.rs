use crate::error::Error;
use git2::{
    Direction, FetchOptions, Oid, Remote,
    build::{CheckoutBuilder, RepoBuilder},
};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use crate::credentials::token_callbacks;

/// Resolve a checkout path within a workspace and verify its Git directory.
pub fn workspace_checkout(workspace: &Path, path: &Path) -> Result<PathBuf, Error> {
    const MISSING_WORKSPACE: Error =
        Error::Unexpected("Run the repository node again to populate its worker workspace");
    let path = if path.is_absolute() {
        path.strip_prefix(workspace).map_err(|_| MISSING_WORKSPACE)?
    } else {
        path
    };
    if path.as_os_str().is_empty()
        || path.components().any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(Error::Unexpected("Invalid source repository path"));
    }
    let workspace = workspace.canonicalize().map_err(|_| MISSING_WORKSPACE)?;
    let repository = workspace.join(path).canonicalize()?;
    let git = repository.join(".git");
    if !repository.starts_with(&workspace) || !git.is_dir() || git.canonicalize()? != git {
        return Err(Error::Unexpected("Source checkout must stay within its workspace"));
    }
    Ok(repository)
}

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

/// Compare committed trees in a local repository, including deleted and renamed paths.
pub fn changed_files(
    path: &Path,
    from_revision: &str,
    to_revision: &str,
    paths: &[String],
) -> Result<Vec<String>, Error> {
    let repository = git2::Repository::open(path)?;
    let old_tree = repository.find_commit(Oid::from_str(from_revision)?)?.tree()?;
    let new_tree = repository.find_commit(Oid::from_str(to_revision)?)?.tree()?;
    let mut options = git2::DiffOptions::new();
    options.disable_pathspec_match(true);
    let all_paths = paths.is_empty()
        || paths.iter().any(|path| {
            Path::new(path).components().all(|component| component == std::path::Component::CurDir)
        });
    if !all_paths {
        for path in paths {
            let path: std::path::PathBuf = Path::new(path)
                .components()
                .filter(|part| *part != std::path::Component::CurDir)
                .collect();
            options.pathspec(path);
        }
    }
    let diff =
        repository.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut options))?;
    let mut files = std::collections::BTreeSet::new();
    for delta in diff.deltas() {
        for path in [delta.old_file().path(), delta.new_file().path()].into_iter().flatten() {
            files.insert(
                path.to_str()
                    .ok_or(Error::Unexpected("Changed file path is not UTF-8"))?
                    .to_owned(),
            );
        }
    }
    Ok(files.into_iter().collect())
}

/// Read the current commit from a local checkout without accessing the remote.
pub fn head(path: &Path) -> Result<String, Error> {
    let repository = git2::Repository::open(path)?;
    Ok(repository.head()?.peel_to_commit()?.id().to_string())
}

/// Read YAML and JSON files from the checked-out commit.
pub fn read_manifests(repository: &Path, path: &Path) -> Result<BTreeMap<String, String>, Error> {
    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => relative.push(part),
            _ => return Err(Error::Unexpected("Manifest path must stay within the repository")),
        }
    }

    let repository = git2::Repository::open(repository)?;
    let root = repository.head()?.peel_to_tree()?;
    let selected = if relative.as_os_str().is_empty() {
        (relative, root.id(), true)
    } else {
        let entry = root.get_path(&relative)?;
        let directory = entry.kind() == Some(git2::ObjectType::Tree);
        if !directory && !is_manifest(&entry, &relative) {
            return Err(Error::Unexpected("Expected a directory or a YAML or JSON file"));
        }
        (relative, entry.id(), directory)
    };

    let odb = repository.odb()?;
    let mut pending = vec![selected];
    let mut manifests = BTreeMap::new();
    let mut entries = 0;
    let mut remaining_bytes = 4 * 1024 * 1024;
    while let Some((path, id, directory)) = pending.pop() {
        if directory {
            let tree = repository.find_tree(id)?;
            entries += tree.len();
            if entries > 4096 {
                return Err(Error::Unexpected("Manifest directory has too many entries"));
            }
            for entry in tree.iter() {
                let child = path.join(entry.name()?);
                let directory = entry.kind() == Some(git2::ObjectType::Tree);
                if directory || is_manifest(&entry, &child) {
                    pending.push((child, entry.id(), directory));
                }
            }
        } else {
            if manifests.len() >= 128 || odb.read_header(id)?.0 > remaining_bytes {
                return Err(Error::Unexpected("Manifests exceed 128 files or 4 MiB"));
            }
            let blob = repository.find_blob(id)?;
            let content = std::str::from_utf8(blob.content())
                .map_err(|_| Error::Unexpected("Manifest is not UTF-8"))?;
            let path = path.to_str().ok_or(Error::Unexpected("Manifest path is not UTF-8"))?;
            remaining_bytes -= blob.size();
            manifests.insert(path.to_owned(), content.to_owned());
        }
    }
    Ok(manifests)
}

fn is_manifest(entry: &git2::TreeEntry<'_>, path: &Path) -> bool {
    matches!(entry.filemode(), 0o100644 | 0o100755)
        && path.extension().is_some_and(|ext| matches!(ext.to_str(), Some("yaml" | "yml" | "json")))
}
