use git2::{Cred, Error, RemoteCallbacks};

pub(crate) fn token_callbacks(mut token: Option<&str>) -> RemoteCallbacks<'_> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_, _, _| match token.take() {
        Some(token) => Cred::userpass_plaintext("oauth2", token),
        None => Err(Error::from_str("Token missing or rejected")),
    });
    callbacks
}
